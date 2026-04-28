using System;
using System.Collections.Generic;
using System.Web.Script.Serialization;

namespace Aiplc.TiaOpenness.Adapter
{
    internal static class Program
    {
        private static readonly JavaScriptSerializer Serializer = CreateSerializer();

        private static int Main(string[] args)
        {
            try
            {
                var bridge = new TiaOpennessBridge(args);
                string line;
                while ((line = Console.In.ReadLine()) != null)
                {
                    if (string.IsNullOrWhiteSpace(line))
                    {
                        continue;
                    }

                    Dictionary<string, object> request;
                    try
                    {
                        request = Serializer.DeserializeObject(line) as Dictionary<string, object>;
                        if (request == null)
                        {
                            throw new InvalidOperationException("Request must be a JSON object.");
                        }
                    }
                    catch (Exception ex)
                    {
                        WriteResponse(
                            FailureResponse(
                                Guid.NewGuid().ToString("N"),
                                "parse_error",
                                ex.Message,
                                new Dictionary<string, object> { { "line", line } }));
                        continue;
                    }

                    WriteResponse(bridge.HandleRequest(request));
                }

                return 0;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine(ex);
                return 1;
            }
        }

        internal static Dictionary<string, object> FailureResponse(
            string id,
            string code,
            string message,
            object details)
        {
            return new Dictionary<string, object>
            {
                { "id", id },
                { "ok", false },
                {
                    "error",
                    new Dictionary<string, object>
                    {
                        { "code", code },
                        { "message", message },
                        { "details", details },
                    }
                },
            };
        }

        internal static Dictionary<string, object> SuccessResponse(string id, object result)
        {
            return new Dictionary<string, object>
            {
                { "id", id },
                { "ok", true },
                { "result", result },
            };
        }

        internal static JavaScriptSerializer CreateSerializer()
        {
            return new JavaScriptSerializer
            {
                MaxJsonLength = int.MaxValue,
                RecursionLimit = 128,
            };
        }

        private static void WriteResponse(Dictionary<string, object> response)
        {
            Console.Out.WriteLine(Serializer.Serialize(response));
            Console.Out.Flush();
        }
    }
}
