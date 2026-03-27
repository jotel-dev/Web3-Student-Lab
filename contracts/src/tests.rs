use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events, Ledger},
    vec, Address, Bytes, Env, FromVal, String, Symbol,
};

fn setup() -> (Env, Address, CertificateContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CertificateContract, ());
    let client = CertificateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init(&admin);
    (env, admin, client)
}

#[test]
fn issues_and_loads_certificate_successfully() {
    let (env, _admin, client) = setup();

    env.ledger().with_mut(|ledger| ledger.timestamp = 1_234);

    let course_symbol = symbol_short!("SOLID");
    let student = Address::generate(&env);
    let course_name = String::from_str(&env, "Rust 101");

    let issued = client.issue(&course_symbol, &vec![&env, student.clone()], &course_name);

    assert_eq!(issued.len(), 1);
    let cert = issued.get(0).unwrap();
    assert_eq!(cert.course_symbol, course_symbol);
    assert_eq!(cert.student, student);
    assert_eq!(cert.course_name, course_name);
    assert_eq!(cert.issue_date, 1_234);
    assert!(!cert.revoked);

    let stored = client.get_certificate(&course_symbol, &student);
    assert_eq!(stored, Some(cert));
}

#[test]
fn returns_none_for_non_existent_certificate() {
    let (env, _admin, client) = setup();

    let course_symbol = symbol_short!("MISSIN");
    let student = Address::generate(&env);
    assert!(client.get_certificate(&course_symbol, &student).is_none());
}

#[test]
fn issues_multiple_students_in_one_call() {
    let (env, _admin, client) = setup();

    env.ledger().with_mut(|ledger| ledger.timestamp = 5_000);

    let course_symbol = symbol_short!("MULTI");
    let course_name = String::from_str(&env, "Web3 Basics");
    let student_a = Address::generate(&env);
    let student_b = Address::generate(&env);
    let student_c = Address::generate(&env);

    let students = vec![
        &env,
        student_a.clone(),
        student_b.clone(),
        student_c.clone(),
    ];
    let issued = client.issue(&course_symbol, &students, &course_name);

    assert_eq!(issued.len(), 3);

    // Each student has a unique, independently stored certificate
    for student in [&student_a, &student_b, &student_c] {
        let cert = client.get_certificate(&course_symbol, student).unwrap();
        assert_eq!(cert.student, *student);
        assert_eq!(cert.issue_date, 5_000);
        assert!(!cert.revoked);
    }
}

#[test]
fn each_student_gets_unique_storage_key() {
    let (env, _admin, client) = setup();

    let course_symbol = symbol_short!("UNIQ");
    let course_name = String::from_str(&env, "Soroban 101");
    let student_a = Address::generate(&env);
    let student_b = Address::generate(&env);

    client.issue(
        &course_symbol,
        &vec![&env, student_a.clone(), student_b.clone()],
        &course_name,
    );

    let cert_a = client.get_certificate(&course_symbol, &student_a).unwrap();
    let cert_b = client.get_certificate(&course_symbol, &student_b).unwrap();
    assert_ne!(cert_a.student, cert_b.student);
}

#[test]
fn verifies_event_emitted_per_student() {
    let (env, _admin, client) = setup();

    let course_symbol = symbol_short!("SOLID");
    let course_name = String::from_str(&env, "Rust 101");
    let student_a = Address::generate(&env);
    let student_b = Address::generate(&env);

    client.issue(
        &course_symbol,
        &vec![&env, student_a.clone(), student_b.clone()],
        &course_name,
    );

    // Count cert_issued events emitted by this contract
    let all_events = env.events().all();
    let mut cert_issued_count = 0u32;
    for (addr, topics, _) in all_events.iter() {
        if addr == client.address
            && Symbol::from_val(&env, &topics.get(0).unwrap()) == Symbol::new(&env, "cert_issued")
        {
            cert_issued_count += 1;
        }
    }

    assert_eq!(cert_issued_count, 2);
}

#[test]
fn admin_can_revoke_certificate() {
    let (env, admin, client) = setup();

    let course_symbol = symbol_short!("SOLID");
    let student = Address::generate(&env);
    let course_name = String::from_str(&env, "Rust 101");

    client.issue(&course_symbol, &vec![&env, student.clone()], &course_name);
    client.revoke(&admin, &course_symbol, &student);

    let cert = client.get_certificate(&course_symbol, &student).unwrap();
    assert!(cert.revoked);
}

#[test]
fn revoke_does_not_affect_other_students() {
    let (env, admin, client) = setup();

    let course_symbol = symbol_short!("SOLID");
    let course_name = String::from_str(&env, "Rust 101");
    let student_a = Address::generate(&env);
    let student_b = Address::generate(&env);

    client.issue(
        &course_symbol,
        &vec![&env, student_a.clone(), student_b.clone()],
        &course_name,
    );

    client.revoke(&admin, &course_symbol, &student_a);

    assert!(
        client
            .get_certificate(&course_symbol, &student_a)
            .unwrap()
            .revoked
    );
    assert!(
        !client
            .get_certificate(&course_symbol, &student_b)
            .unwrap()
            .revoked
    );
}

#[test]
fn revoke_emits_event() {
    let (env, admin, client) = setup();

    let course_symbol = symbol_short!("SOLID");
    let student = Address::generate(&env);
    let course_name = String::from_str(&env, "Rust 101");

    client.issue(&course_symbol, &vec![&env, student.clone()], &course_name);
    client.revoke(&admin, &course_symbol, &student);

    let (addr, topics, _data) = env.events().all().last().unwrap();
    assert_eq!(addr, client.address);
    assert_eq!(
        Symbol::from_val(&env, &topics.get(0).unwrap()),
        Symbol::new(&env, "cert_revoked")
    );
    assert_eq!(
        Symbol::from_val(&env, &topics.get(1).unwrap()),
        course_symbol
    );
}

#[test]
#[should_panic(expected = "unauthorized")]
fn non_admin_cannot_revoke_certificate() {
    let (env, _admin, client) = setup();

    let course_symbol = symbol_short!("SOLID");
    let student = Address::generate(&env);
    let course_name = String::from_str(&env, "Rust 101");

    client.issue(&course_symbol, &vec![&env, student.clone()], &course_name);

    let attacker = Address::generate(&env);
    client.revoke(&attacker, &course_symbol, &student);
}

// ---------------------------------------------------------------------------
// #162 – Meta-Transaction tests
// ---------------------------------------------------------------------------

#[test]
fn meta_tx_issues_certificate_for_student() {
    let (env, admin, client) = setup();

    env.ledger().with_mut(|l| l.timestamp = 9_000);

    let course_symbol = symbol_short!("META");
    let student = Address::generate(&env);
    let course_name = String::from_str(&env, "Meta Course");

    let call_data = MetaTxCallData {
        course_symbol: course_symbol.clone(),
        student: student.clone(),
        course_name: course_name.clone(),
        nonce: 0,
    };

    let cert = client.execute_meta_tx(&admin, &Bytes::new(&env), &call_data);

    assert_eq!(cert.student, student);
    assert_eq!(cert.course_symbol, course_symbol);
    assert_eq!(cert.issue_date, 9_000);
    assert!(!cert.revoked);

    // Certificate is retrievable
    assert_eq!(
        client.get_certificate(&course_symbol, &student),
        Some(cert)
    );
}

#[test]
fn meta_tx_nonce_increments_after_execution() {
    let (env, admin, client) = setup();

    let call_data = MetaTxCallData {
        course_symbol: symbol_short!("NONCE"),
        student: Address::generate(&env),
        course_name: String::from_str(&env, "Nonce Test"),
        nonce: 0,
    };

    assert_eq!(client.get_nonce(&admin), 0);
    client.execute_meta_tx(&admin, &Bytes::new(&env), &call_data);
    assert_eq!(client.get_nonce(&admin), 1);
}

#[test]
#[should_panic(expected = "invalid nonce")]
fn meta_tx_replay_is_rejected() {
    let (env, admin, client) = setup();

    let call_data = MetaTxCallData {
        course_symbol: symbol_short!("REPLY"),
        student: Address::generate(&env),
        course_name: String::from_str(&env, "Replay Test"),
        nonce: 0,
    };

    client.execute_meta_tx(&admin, &Bytes::new(&env), &call_data.clone());
    // Second call with same nonce must panic
    client.execute_meta_tx(&admin, &Bytes::new(&env), &call_data);
}

#[test]
#[should_panic(expected = "unauthorized")]
fn meta_tx_non_admin_is_rejected() {
    let (env, _admin, client) = setup();

    let attacker = Address::generate(&env);
    let call_data = MetaTxCallData {
        course_symbol: symbol_short!("HACK"),
        student: Address::generate(&env),
        course_name: String::from_str(&env, "Hack Attempt"),
        nonce: 0,
    };

    client.execute_meta_tx(&attacker, &Bytes::new(&env), &call_data);
}

#[test]
fn meta_tx_emits_event() {
    let (env, admin, client) = setup();

    let course_symbol = symbol_short!("EVNT");
    let call_data = MetaTxCallData {
        course_symbol: course_symbol.clone(),
        student: Address::generate(&env),
        course_name: String::from_str(&env, "Event Test"),
        nonce: 0,
    };

    client.execute_meta_tx(&admin, &Bytes::new(&env), &call_data);

    let (addr, topics, _) = env.events().all().last().unwrap();
    assert_eq!(addr, client.address);
    assert_eq!(
        Symbol::from_val(&env, &topics.get(0).unwrap()),
        Symbol::new(&env, "meta_tx_issued")
    );
}
