import UserModel, {IUserDocument} from "./models/User";
import * as jwt from "jsonwebtoken";

interface IUserPayload {
	sub: string,
	name: string
}

export async function validateUser(req, res, next){
	let userToken = req.cookies?.user;
	if (userToken === undefined){
		let token = await newUserToken();
		res.cookie("user", token, {sameSite: true});
	} else {
		let payload: IUserPayload;
		try{
			payload = jwt.verify(userToken, "testing, remove me!");
		} catch (e) {
			let token = await newUserToken();
			next();
			return;
		}
		
		let userEntity = await UserModel.findOne({uuid: payload.sub});

		if (userEntity === null){
			let token = await newUserToken();
			res.cookie("user", token, {sameSite: true});
		} else if (userEntity.name !== payload.name){
			let token = await signJWT(userEntity);
			res.cookie("user", token, {sameSite: true});
		}
	}

	next();
}

async function signJWT(user: IUserDocument){
	return jwt.sign({
		sub: user.uuid,
		name: user.name
	}, "testing, remove me!");
}

async function newUserToken() {
	const newUser = new UserModel();
	await newUser.save();

	return signJWT(newUser);
}