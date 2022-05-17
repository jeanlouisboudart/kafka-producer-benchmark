using System;
using Microsoft.Extensions.Logging;
using ILogger = Microsoft.Extensions.Logging.ILogger;
using ILoggerFactory = Microsoft.Extensions.Logging.ILoggerFactory;
public static class Logger
    {
        private static ILoggerFactory _factory;

        /// <summary>
        /// If logger factory is not set by the project using the library
        /// it will create a factory with console logger configured and minimum log level to Information.
        /// </summary>
        internal static ILoggerFactory LoggerFactory
        {
            get => _factory ??= Microsoft.Extensions.Logging.LoggerFactory.Create(builder =>
            {
                builder.SetMinimumLevel(LogLevel.Information);
                builder.AddSimpleConsole(options =>
                {
                    options.IncludeScopes = true;
                    options.SingleLine = true;
                    options.TimestampFormat = "[yyyy-MM-dd HH:mm:ss] ";
                });
                builder.AddConsole();
            });
            set => _factory = value;
        }

        /// <summary>   
        /// Get logger from type class.
        /// </summary>
        /// <param name="type">Class type which call logger</param>
        /// <returns>Return logger configured</returns>
        public static ILogger GetLogger(Type type)
        {
            return LoggerFactory.CreateLogger(type);
        }
    }