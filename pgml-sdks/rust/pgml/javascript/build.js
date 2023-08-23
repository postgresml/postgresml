const os = require("os");
const { exec } = require("node:child_process");

const type = os.type();
const arch = os.arch();

const set_name = (type, arch) => {
  if (type == "Darwin" && arch == "x64") {
    return "x86_64-apple-darwin-index.node";
  } else if (type == "Darwin" && arch == "arm64") {
    return "aarch64-apple-darwin-index.node";
  } else if ((type == "Windows" || type == "Windows_NT") && arch == "x64") {
    return "x86_64-pc-windows-gnu-index.node";
  } else if (type == "Linux" && arch == "x64") {
    return "x86_64-unknown-linux-gnu-index.node";
  } else if (type == "Linux" && arch == "arm64") {
    return "aarch64-unknown-linux-gnu-index.node";
  } else {
    console.log("UNSUPPORTED TYPE OR ARCH:", type, arch);
    process.exit(1);
  }
};

let name = set_name(type, arch);

let args = process.argv.slice(2);
let release = args.includes("--release");

let shell_args =
  type == "Windows" || type == "Windows_NT" ? { shell: "powershell.exe" } : {};

exec(
  `
  rm -r dist;
  mkdir dist;
  npx cargo-cp-artifact -nc "${name}" -- cargo build --message-format=json-render-diagnostics -F javascript ${release ? "--release" : ""};
  mv ${name} dist;
  `,
  shell_args,
  (err, stdout, stderr) => {
    if (err) {
      console.log("ERR:", err);
    } else {
      console.log("STDOUT:", stdout);
      console.log("STDERR:", stderr);
    }
  },
);
