const os = require("os")

const type = os.type()
const arch = os.arch()

// if (type == "Darwin" && arch == "arm64") {
// 	const pgml = require("./index.node")
// 	module.exports = pgml
// } else {
// 	console.log("UNSUPPORTED TYPE OR ARCH:", type, arch)
// }


const pgml = require("./index.node")
module.exports = pgml
