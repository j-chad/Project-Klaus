import mongoose from "mongoose";

export enum APIResponseStatus {
	Success = "success",
	Fail = "fail",
	Error = "error"
}


// JSend protocol wrapper
export function APIResponse(status = APIResponseStatus.Success, data: any = null) {
	if (status === APIResponseStatus.Error){
		return {
			status: status,
			message: data
		}
	} else {
		return {
			status: status,
			data: data
		}
	}
}