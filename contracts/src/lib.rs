#![no_std]

pub mod token;

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Certificate {
    pub course_symbol: Symbol,
    pub student: Address,
    pub course_name: String,
    pub issue_date: u64,
    pub revoked: bool,
}

/// Composite storage key: one entry per (course, student) pair.
#[contracttype]
#[derive(Clone)]
pub struct CertKey {
    pub course_symbol: Symbol,
    pub student: Address,
}

/// Encoded call data for a meta-transaction.
/// Represents an `issue` call on behalf of a student.
#[contracttype]
#[derive(Clone)]
pub struct MetaTxCallData {
    pub course_symbol: Symbol,
    pub student: Address,
    pub course_name: String,
    /// Nonce prevents replay attacks.
    pub nonce: u64,
}

const ADMIN_KEY: &str = "admin";
const NONCE_PREFIX: &str = "nonce";

#[contract]
pub struct CertificateContract;

#[contractimpl]
impl CertificateContract {
    /// Initialize the contract with an administrator address.
    /// Must be called once before any other function.
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&Symbol::new(&env, ADMIN_KEY)) {
            panic!("already initialized");
        }
        env.storage()
            .instance()
            .set(&Symbol::new(&env, ADMIN_KEY), &admin);
    }

    /// Issue certificates to one or more students for a course in a single transaction.
    /// Each student gets a unique certificate stored under (course_symbol, student) key.
    /// A `cert_issued` event is emitted for every certificate.
    pub fn issue(
        env: Env,
        course_symbol: Symbol,
        students: Vec<Address>,
        course_name: String,
    ) -> Vec<Certificate> {
        let issue_date = env.ledger().timestamp();
        let mut issued: Vec<Certificate> = Vec::new(&env);

        for student in students.iter() {
            let key = CertKey {
                course_symbol: course_symbol.clone(),
                student: student.clone(),
            };

            let cert = Certificate {
                course_symbol: course_symbol.clone(),
                student: student.clone(),
                course_name: course_name.clone(),
                issue_date,
                revoked: false,
            };

            env.storage().instance().set(&key, &cert);

            env.events().publish(
                (Symbol::new(&env, "cert_issued"), course_symbol.clone()),
                (student.clone(), course_name.clone()),
            );

            issued.push_back(cert);
        }

        issued
    }

    /// Revoke a certificate by course symbol and student address.
    /// Only callable by the administrator.
    pub fn revoke(env: Env, caller: Address, course_symbol: Symbol, student: Address) {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, ADMIN_KEY))
            .expect("contract not initialized");

        if caller != admin {
            panic!("unauthorized: only the admin can revoke certificates");
        }

        let key = CertKey {
            course_symbol: course_symbol.clone(),
            student: student.clone(),
        };

        let mut cert: Certificate = env
            .storage()
            .instance()
            .get(&key)
            .expect("certificate not found");

        cert.revoked = true;
        env.storage().instance().set(&key, &cert);

        env.events().publish(
            (Symbol::new(&env, "cert_revoked"), course_symbol),
            (caller, student),
        );
    }

    /// Retrieve a certificate by course symbol and student address.
    pub fn get_certificate(
        env: Env,
        course_symbol: Symbol,
        student: Address,
    ) -> Option<Certificate> {
        let key = CertKey {
            course_symbol,
            student,
        };
        env.storage().instance().get(&key)
    }

    // -------------------------------------------------------------------------
    // #162 – Gas-Free Meta Transactions
    // -------------------------------------------------------------------------

    /// Execute a meta-transaction: the instructor pays the fee while the
    /// certificate is issued on behalf of the student.
    ///
    /// `instructor`  – the address that signs and pays for the transaction.
    /// `signature`   – Ed25519 signature over the XDR-encoded `call_data`.
    /// `call_data`   – the operation to execute (issue a certificate).
    ///
    /// Security:
    /// - The instructor must be the contract admin.
    /// - A per-instructor nonce prevents replay attacks.
    /// - `require_auth` enforces on-chain signature verification by the Soroban host.
    pub fn execute_meta_tx(
        env: Env,
        instructor: Address,
        _signature: Bytes,
        call_data: MetaTxCallData,
    ) -> Certificate {
        // Require the instructor to authorise this invocation (host verifies sig).
        instructor.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, ADMIN_KEY))
            .expect("contract not initialized");

        if instructor != admin {
            panic!("unauthorized: only the admin instructor can submit meta-transactions");
        }

        // Replay protection: nonce must match the stored value (default 0).
        let nonce_key = (Symbol::new(&env, NONCE_PREFIX), instructor.clone());
        let stored_nonce: u64 = env
            .storage()
            .instance()
            .get(&nonce_key)
            .unwrap_or(0u64);

        if call_data.nonce != stored_nonce {
            panic!("invalid nonce");
        }

        // Advance nonce before executing to prevent reentrancy-style replays.
        env.storage()
            .instance()
            .set(&nonce_key, &(stored_nonce + 1));

        // Issue the certificate for the student.
        let issue_date = env.ledger().timestamp();
        let key = CertKey {
            course_symbol: call_data.course_symbol.clone(),
            student: call_data.student.clone(),
        };

        let cert = Certificate {
            course_symbol: call_data.course_symbol.clone(),
            student: call_data.student.clone(),
            course_name: call_data.course_name.clone(),
            issue_date,
            revoked: false,
        };

        env.storage().instance().set(&key, &cert);

        env.events().publish(
            (
                Symbol::new(&env, "meta_tx_issued"),
                call_data.course_symbol.clone(),
            ),
            (instructor, call_data.student.clone(), call_data.course_name),
        );

        cert
    }

    /// Return the current meta-transaction nonce for an instructor address.
    pub fn get_nonce(env: Env, instructor: Address) -> u64 {
        let nonce_key = (Symbol::new(&env, NONCE_PREFIX), instructor);
        env.storage()
            .instance()
            .get(&nonce_key)
            .unwrap_or(0u64)
    }

    // -------------------------------------------------------------------------
    // #163 – Contract Upgradeability (Proxy Pattern)
    // -------------------------------------------------------------------------

    /// Upgrade the contract WASM to a new version identified by `new_wasm_hash`.
    ///
    /// Restricted to the ADMIN address only.
    ///
    /// Security considerations:
    /// - Only the admin can trigger an upgrade, preventing unauthorised code replacement.
    /// - The new WASM hash must already be uploaded to the network before calling this.
    /// - All existing instance storage (certificates, nonces, admin key) is preserved
    ///   across upgrades because Soroban keeps instance storage independent of WASM.
    /// - After upgrading, verify the new contract behaves correctly before revoking
    ///   the old admin key or migrating state.
    /// - A compromised admin key means a compromised upgrade path – use a multisig
    ///   or timelock for production deployments.
    pub fn upgrade(env: Env, caller: Address, new_wasm_hash: BytesN<32>) {
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, ADMIN_KEY))
            .expect("contract not initialized");

        if caller != admin {
            panic!("unauthorized: only the admin can upgrade the contract");
        }

        env.deployer()
            .update_current_contract_wasm(new_wasm_hash);
    }
}

#[cfg(test)]
mod tests;
