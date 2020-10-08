import winston from "winston";
import appRoot from "app-root-path";

const logger = winston.createLogger({
	transports: [
		new winston.transports.File({
			filename: `${appRoot}/logs/error.log`,
			level: 'error',
			handleExceptions: true,
			maxsize: 5242880, // 5MB
			maxFiles: 5,
			format: winston.format.json()
		}),
		new winston.transports.File({
			filename: `${appRoot}/logs/http.log`,
			level: "http",
			handleExceptions: true,
			maxsize: 5242880, // 5MB
			maxFiles: 5,
			format: winston.format.json()
		})
	],
});

//
// If we're not in production then log to the `console`
//
if (process.env.NODE_ENV !== 'production') {
	logger.add(new winston.transports.Console({
		level: 'debug',
		handleExceptions: true,
		format: winston.format.combine(
			winston.format.colorize(),
			winston.format.simple()
		)
	}));
}

export default logger;