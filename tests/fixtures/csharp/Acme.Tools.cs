using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Net.Http;
using System.Runtime.InteropServices;

namespace Acme.Tools
{
    public static class Shell
    {
        public static void Run(int mode)
        {
            switch (mode)
            {
                case 3: Console.WriteLine(1L << 40); break;
                case 4: Console.WriteLine(2.5); break;
                case 5: Console.WriteLine("five"); break;
            }
            Process.Start("ls");
        }
    }

    public interface IRunner
    {
        void Go();
    }

    public class Runner : IRunner
    {
        public void Go()
        {
            Shell.Run(0);
        }
    }

    public class Caller
    {
        public void Call(IRunner runner)
        {
            runner.Go();
            var items = new List<string>();
            items.Add("x");
            Action later = () => Shell.Run(1);
            later();
        }
    }

    public class Fetcher
    {
        public void Get()
        {
            new HttpClient().GetStringAsync("https://example.com").Wait();
        }
    }

    public static class Native
    {
        [DllImport("libc")]
        private static extern int getpid();

        public static int Pid()
        {
            return getpid();
        }
    }

    public static class Settings
    {
        static Settings()
        {
            Shell.Run(2);
        }

        public static void Touch()
        {
        }
    }

    public class Outer
    {
        public class Inner
        {
            public void Work()
            {
                Shell.Run(3);
            }
        }
    }
}
