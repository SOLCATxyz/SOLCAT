import type { NextPage } from 'next';
import Head from 'next/head';
import React, { useEffect, useState } from 'react';

const Home: NextPage = () => {
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    // Initialize app
    setIsLoading(false);
  }, []);

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <Head>
        <title>SOLCAT - Solana Ecosystem Guardian</title>
        <meta name="description" content="SOLCAT is a revolutionary Solana ecosystem guardian, combining meme culture with practical utility to combat airdrop hunters and protect the ecosystem's fairness." />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <main className="container mx-auto px-4 py-16">
        <div className="text-center">
          <h1 className="text-6xl font-bold mb-8">
            Welcome to SOLCAT
          </h1>
          <p className="text-xl mb-8">
            The Guardian of Solana Ecosystem
          </p>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mt-16">
            <div className="p-6 bg-gray-800 rounded-lg">
              <h2 className="text-2xl font-bold mb-4">CatEye Browser Extension</h2>
              <p>Mark and monitor suspicious addresses in the Solana ecosystem</p>
            </div>
            <div className="p-6 bg-gray-800 rounded-lg">
              <h2 className="text-2xl font-bold mb-4">CatPaw Analytics</h2>
              <p>Advanced dashboard for tracking and analyzing suspicious activities</p>
            </div>
            <div className="p-6 bg-gray-800 rounded-lg">
              <h2 className="text-2xl font-bold mb-4">CatCouncil DAO</h2>
              <p>Community-driven governance for ecosystem protection</p>
            </div>
          </div>
        </div>
      </main>

      <footer className="container mx-auto px-4 py-8 mt-16 border-t border-gray-800">
        <div className="flex justify-between items-center">
          <div>© 2024 SOLCAT</div>
          <div className="flex space-x-6">
            <a href="https://www.solcat.work" className="hover:text-yellow-500">Website</a>
            <a href="https://x.com/SOLCAT_xyz" className="hover:text-yellow-500">Twitter</a>
            <a href="https://github.com/SOLCATxyz/SOLCAT" className="hover:text-yellow-500">GitHub</a>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Home; 