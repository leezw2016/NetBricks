using System;
using E2D2.SNApi;
using E2D2;
using E2D2.Collections;
using System.Runtime.CompilerServices; 
using System.IO;
using System.Net;
namespace E2D2 {
	public sealed class FixedGCTest {
		public static void Main(string[] args) {
			Console.WriteLine("Begin parsing arguments");
			var options = E2D2OptionParser.ParseOptions(args);
			int nrxq = options.numRxq;
			int ntxq = options.numTxq;
			Console.WriteLine("Done parsing arguments");
			Console.WriteLine("Found rxq {0} txq {1}", nrxq, ntxq);
			IE2D2Component vf1 = new BaseLineVF();
			SoftNic.init_softnic (2, "test");
			IntPtr port1 = SoftNic.init_port ("vport0");
			IntPtr port2 = SoftNic.init_port ("vport1");
			PacketBuffer pkts = SoftNic.CreatePacketBuffer(32);
			Console.WriteLine("DPDK LCORE setting {0}", SoftNic.sn_get_lcore_id());
			int pollRx= 0;
			int pollTx = 0;
			while (true) {
				int rcvd = SoftNic.ReceiveBatch(port1, pollRx, ref pkts);
				pollRx = (pollRx + 1) % nrxq;
				try {
					vf1.PushBatch(ref pkts);
				} catch (Exception) {
				}
				if (rcvd > 0) {
					SoftNic.SendBatch(port2, pollTx, ref pkts);
				}
				pollTx = (pollTx + 1) % ntxq; 
			}
		}
	}
}