const path = require('path');
const util = require("util");
const exec = util.promisify(require("child_process").exec);

const yaml = require("js-yaml");
const fs = require("fs");
// const readdir = util.promisify(fs.readdir);


const { TestRunner } = require('./test-runner');


(async function () {
	process.on('unhandledRejection', (error) => {
		console.error(error);
		process.exit(1);
	});

	main(parse_args()).catch((error) => {
		console.error(error);
		process.exit(1);
	});
})();


function parse_args() {
	const { argv } = require("yargs");
	let cases_dir = argv._[0] ? argv._[0] : null;
	if (!cases_dir) {
		console.error("Missing cases-directory argument...");
		process.exit();
	}
	cases_dir = path.resolve(process.cwd(), cases_dir);
	if (!fs.existsSync(cases_dir)) {
		console.error("Cases directory does not exist: ", cases_dir);
		process.exit();
	}
	return { cases_dir, };
}


async function main({ cases_dir }) {
	// TODO: let {err, error, files}
	let files = fs.readdirSync(cases_dir).filter(
		(value, i, foo) => (value.toLowerCase().endsWith(".yaml") || value.toLowerCase().endsWith(".yml") ? value : null)
	);

	// console.log(files);

	for (let file of files) {
		// console.log("loading case:", file);

		try {
			const tasks = yaml.load(fs.readFileSync(path.join(cases_dir, file), "utf8"));

			// phase: Setup
			const { setup } = require("./setup");
			const runner_setup = await setup(tasks.setup);

			if (!runner_setup || !runner_setup.result?.success) {
				console.error("case setup failed", file);
				continue;
			}

			// phase: Tests
			if (tasks.tests?.length == 0) {
				console.warn("case have no tests");
				continue;
			}

			const fail_fast = true;

			const on_success = (test_i, step_i) => {
				const test = tasks.tests[test_i];
				const step = test.steps[step_i];
				console.log("ON_SUCCESS", (test_i, step_i), test.name, step.name);
			};
			const on_fail = (test_i, step_i, message) => {
				const test = tasks.tests[test_i];
				const step = test.steps[step_i];
				console.log("ON_FAIL", (test_i, step_i), test.name, step.name);

				if (fail_fast)
					process.exit(1);
			};

			// for (let test of tasks.tests) {
			for (const i in tasks.tests) {
				const test = tasks.tests[i];
				console.log(`Running test ${i}:`, test.name);

				const runner = new TestRunner(
					test,
					runner_setup.result.nodes,
					(step_i) => { on_success(i, step_i) },
					(step_i, message) => { on_fail(i, step_i, message) },
					(...rest) => { console.debug("*", ...rest) }
				);
				await runner.run();
			}

		} catch (error) {
			console.error(error);
			continue;
		}
	}
}
