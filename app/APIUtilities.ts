export enum APIResponseStatus {
	Success = "success",
	Fail = "fail",
	Error = "error"
}

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