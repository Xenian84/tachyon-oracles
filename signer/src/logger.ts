import winston from 'winston';

// Helper to safely stringify objects with circular references
function safeStringify(obj: any): string {
  try {
    return JSON.stringify(obj, (key, value) => {
      // Skip circular references and large objects
      if (key === 'req' || key === 'res' || key === 'socket') {
        return '[Circular]';
      }
      return value;
    });
  } catch (e) {
    return '[Unable to stringify]';
  }
}

export const logger = winston.createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.json()
  ),
  transports: [
    new winston.transports.Console({
      format: winston.format.combine(
        winston.format.colorize(),
        winston.format.printf(({ timestamp, level, message, ...meta }) => {
          return `${timestamp} [${level}]: ${message} ${Object.keys(meta).length ? safeStringify(meta) : ''}`;
        })
      ),
    }),
    new winston.transports.File({ filename: 'signer-error.log', level: 'error' }),
    new winston.transports.File({ filename: 'signer.log' }),
  ],
});

