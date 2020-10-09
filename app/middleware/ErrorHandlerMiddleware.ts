import {Response, Request} from "express";
import mongoose, {Error} from "mongoose";
import {APIResponse, APIResponseStatus} from "../helpers/APIUtilities";

export default async function HandleError(err: Error, req: Request, res: Response, next) {
	if (res.headersSent) {
		return next(err)
	}

	if (err instanceof mongoose.Error.ValidationError){
		handleValidationError(res, err);
	} else {
		handleUndefinedError(err);
	}
}

function handleValidationError(res: Response, e: mongoose.Error.ValidationError) {
	let data = {};
	let errors = e.errors;
	Object.keys(errors).forEach((key) => {
		let error = errors[key];
		data[key] = {
			"kind": error.kind,
			"path": error.path,
			"value": error.value,
			"msg": error.message,
		}
	});

	res.status(400);
	res.json(APIResponse(APIResponseStatus.Fail, data));
}

function handleUndefinedError(res) {
	res.status(500);
	res.json(APIResponse(APIResponseStatus.Error, "Unknown Error"));
}