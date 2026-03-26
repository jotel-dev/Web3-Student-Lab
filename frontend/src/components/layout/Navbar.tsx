'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { useState } from 'react';

export default function Navbar() {
  const pathname = usePathname();
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);

  const isActive = (path: string) => pathname === path;

  return (
    <nav className="sticky top-0 z-50 w-full border-b border-white/10 bg-black/80 backdrop-blur-md">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between items-center h-20">
          
          {/* Logo */}
          <div className="flex items-center gap-3">
            <Link href="/" className="flex items-center gap-3 group">
              <div className="w-10 h-10 bg-red-600 rounded-lg flex items-center justify-center transform group-hover:rotate-12 transition-transform duration-300 shadow-[0_0_15px_rgba(220,38,38,0.5)]">
                <svg className="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.747 0 3.332.477 4.5 1.253v13C19.832 18.477 18.247 18 16.5 18c-1.746 0-3.332.477-4.5 1.253" />
                </svg>
              </div>
              <span className="text-2xl font-black text-white tracking-widest uppercase">
                Web3 <span className="text-red-600">Lab</span>
              </span>
            </Link>
          </div>
          
          {/* Desktop Navigation */}
          <div className="hidden md:flex items-center gap-8">
            <Link
              href="/courses"
              className={`text-sm font-bold tracking-wide transition-colors ${
                isActive('/courses') ? 'text-red-500' : 'text-gray-300 hover:text-white'
              }`}
            >
              MODULES
            </Link>
            <Link
              href="/verify"
              className={`text-sm font-bold tracking-wide transition-colors ${
                isActive('/verify') ? 'text-red-500' : 'text-gray-300 hover:text-white'
              }`}
            >
              VERIFY
            </Link>
            <div className="w-px h-6 bg-white/20"></div>
            <Link
              href="/auth/login"
              className="text-sm font-bold tracking-wide text-gray-300 hover:text-white transition-colors"
            >
              SIGN IN
            </Link>
            <Link
              href="/auth/register"
              className="px-6 py-2.5 bg-red-600 hover:bg-red-700 text-white text-sm font-bold tracking-wide rounded border border-red-500 shadow-[0_0_15px_rgba(220,38,38,0.4)] hover:shadow-[0_0_25px_rgba(220,38,38,0.6)] transition-all uppercase"
            >
              Initialize Node
            </Link>
          </div>

          {/* Mobile Menu Button */}
          <div className="md:hidden flex items-center">
            <button
              onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
              className="text-gray-300 hover:text-white focus:outline-none"
            >
              <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                {isMobileMenuOpen ? (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                ) : (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                )}
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Menu Panel */}
      {isMobileMenuOpen && (
        <div className="md:hidden bg-black border-b border-white/10 px-4 pt-2 pb-6 space-y-4 shadow-2xl">
          <Link
            href="/courses"
            onClick={() => setIsMobileMenuOpen(false)}
            className="block px-3 py-2 text-base font-bold text-gray-300 hover:text-white hover:bg-white/5 rounded-md uppercase"
          >
            Modules
          </Link>
          <Link
            href="/verify"
            onClick={() => setIsMobileMenuOpen(false)}
            className="block px-3 py-2 text-base font-bold text-gray-300 hover:text-white hover:bg-white/5 rounded-md uppercase"
          >
            Verify Certificate
          </Link>
          <div className="h-px w-full bg-white/10 my-2"></div>
          <Link
            href="/auth/login"
            onClick={() => setIsMobileMenuOpen(false)}
            className="block px-3 py-2 text-base font-bold text-gray-300 hover:text-white hover:bg-white/5 rounded-md uppercase"
          >
            Sign In
          </Link>
          <Link
            href="/auth/register"
            onClick={() => setIsMobileMenuOpen(false)}
            className="block px-3 py-2 text-base font-bold text-red-500 hover:text-red-400 hover:bg-red-500/10 rounded-md uppercase"
          >
            Initialize Node (Register)
          </Link>
        </div>
      )}
    </nav>
  );
}
