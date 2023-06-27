var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
import pgml from '../../index.js';
const CONNECTION_STRING = process.env.DATABASE_URL ? process.env.DATABASE_URL : "";
function test() {
    return __awaiter(this, void 0, void 0, function* () {
        let db = yield pgml.newDatabase(CONNECTION_STRING);
        let collection_name = "ttest2";
        let collection = yield db.create_or_get_collection(collection_name);
        console.log(collection);
    });
}
test().then(() => console.log("\nTests Done!")).catch((err) => console.log(err));
