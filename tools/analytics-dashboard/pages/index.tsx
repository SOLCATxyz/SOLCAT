import { NextPage } from 'next';
import Head from 'next/head';
import { Card, Title, AreaChart, DonutChart, BarList, Flex, Grid } from '@tremor/react';
import { useState, useEffect } from 'react';

const Dashboard: NextPage = () => {
  const [isLoading, setIsLoading] = useState(true);
  const [stats, setStats] = useState({
    totalScanned: 0,
    totalReported: 0,
    riskDistribution: [],
    activityTimeline: [],
    topSuspiciousAddresses: []
  });

  useEffect(() => {
    fetchDashboardData();
  }, []);

  const fetchDashboardData = async () => {
    try {
      const response = await fetch('https://api.solcat.work/dashboard/stats');
      const data = await response.json();
      setStats(data);
    } catch (error) {
      console.error('Error fetching dashboard data:', error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gray-900">
      <Head>
        <title>SOLCAT Analytics Dashboard</title>
        <meta name="description" content="SOLCAT Analytics Dashboard - Monitor suspicious activities in the Solana ecosystem" />
      </Head>

      <main className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-white mb-2">SOLCAT Analytics</h1>
          <p className="text-gray-400">Monitor and analyze suspicious activities in real-time</p>
        </div>

        <Grid numItems={1} numItemsSm={2} numItemsLg={4} className="gap-6 mb-6">
          <Card className="bg-gray-800">
            <Title className="text-white">Total Addresses Scanned</Title>
            <p className="text-2xl font-bold text-yellow-500 mt-2">{stats.totalScanned}</p>
          </Card>
          <Card className="bg-gray-800">
            <Title className="text-white">Reported Addresses</Title>
            <p className="text-2xl font-bold text-red-500 mt-2">{stats.totalReported}</p>
          </Card>
          <Card className="bg-gray-800">
            <Title className="text-white">Risk Distribution</Title>
            <DonutChart
              data={stats.riskDistribution}
              category="value"
              index="name"
              colors={["emerald", "yellow", "red"]}
              className="mt-4 h-32"
            />
          </Card>
          <Card className="bg-gray-800">
            <Title className="text-white">Activity Timeline</Title>
            <AreaChart
              data={stats.activityTimeline}
              index="date"
              categories={["scanned", "reported"]}
              colors={["blue", "red"]}
              className="mt-4 h-32"
            />
          </Card>
        </Grid>

        <Grid numItems={1} numItemsLg={2} className="gap-6">
          <Card className="bg-gray-800">
            <Title className="text-white">Top Suspicious Addresses</Title>
            <BarList
              data={stats.topSuspiciousAddresses}
              className="mt-4"
            />
          </Card>
          <Card className="bg-gray-800">
            <Title className="text-white">Recent Activity</Title>
            <div className="mt-4">
              {/* Add activity feed component here */}
            </div>
          </Card>
        </Grid>
      </main>

      <footer className="container mx-auto px-4 py-8 mt-8 border-t border-gray-800">
        <Flex justifyContent="between" className="text-gray-400">
          <p>© 2024 SOLCAT</p>
          <div className="space-x-4">
            <a href="https://www.solcat.work" className="hover:text-yellow-500">Website</a>
            <a href="https://x.com/SOLCAT_xyz" className="hover:text-yellow-500">Twitter</a>
            <a href="https://github.com/SOLCATxyz/SOLCAT" className="hover:text-yellow-500">GitHub</a>
          </div>
        </Flex>
      </footer>
    </div>
  );
};

export default Dashboard; 