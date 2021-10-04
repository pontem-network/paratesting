const { ApiPromise, SubmittableResult, WsProvider } = require("@polkadot/api");
const { ApiOptions } = require("@polkadot/api/types");
const { Keyring } = require("@polkadot/keyring");
const { assert, isFunction, stringify } = require("@polkadot/util");


class TestContext {
	#success; #failure; #log;
	test;
	step;
	temp;

	constructor(test, step, success, failure, log) {
		this.test = test;
		this.step = step;
		this.temp = {};
		this.#success = success;
		this.#failure = failure;
		this.#log = log;
	}

	success() { this.#success() }
	fail(message) { this.#failure(message) }
	log(...args) { this.#log(...args) }
	debug(...args) { this.#log(...args) }
	error(...args) { this.#log("ERROR", ...args) }
}


class TestRunner {
	#cfg; #nodes;
	#on_step_success; #on_step_fail; #log;

	constructor(test, nodes, on_step_success, on_step_fail, log) {
		this.#cfg = test;
		this.#nodes = nodes;
		this.#on_step_success = on_step_success;
		this.#on_step_fail = on_step_fail;
		this.#log = log;
	}

	get steps() { return this.#cfg.steps; }
	step(i) { return this.#cfg.steps[i]; }

	async run() {

		let current = Promise.resolve();
		for (const i in this.#cfg.steps) {
			const step = this.#cfg.steps[i];
			const ctx = new TestContext(
				this.#cfg,
				step,
				(/* node */) => { this.#on_step_success(i) },
				(/* node, */ message) => { this.#on_step_fail(i, message) },
				(...args) => { this.#log(`[${i}]`, ...args) },
			);

			ctx.debug(`\t step ${i} :`, step);
			current = current.then(this.run_step(i, step, ctx));
		}
		await current;
	}


	async run_step(i, step, step_ctx) {
		const nodes = this.get_nodes_for_step(step);
		step_ctx.debug("\tusing nodes:", Object.keys(nodes));

		let all = [];
		for (const key in nodes) {
			const node = nodes[key];
			const ctx = new TestContext(
				this.#cfg,
				step,
				() => { step_ctx.success() },
				(message) => { step_ctx.fail(message) },
				(...args) => { step_ctx.log(`${key}`, ...args) },
			);

			all.push(this.run_step_for(i, ctx, step, node));
		}
		return Promise.all(all);
	}


	async run_step_for(i, ctx, step, node) {
		ctx.debug("make call for step:", (i), step.name);

		const call_info = await make_call_info(step, node, ctx, this);

		const method = (() => {
			let method = send_call;
			switch (call_info.type) {
				case "consts":
					method = query_const;
					break;
				case "tx":
					method = send_tx;
					break;
				default:
					method = send_call;
					break;
			}
			return method;
		})();

		const check = this.make_check(step, ctx);

		return /* await */ method(ctx, call_info).then(check);
		// return /* await */ method(ctx, call_info);
		//.then((res) => { console.debug(`RESULT of ${i}`, res); res });
	}


	make_check(step, ctx) {
		let f = (any) => { any };
		if (step.success?.result) {
			const resolve_key = require('object-resolve-path');

			const expected = step.success.result;
			f = (res) => {
				const status = {
					ready: res.isReady,
					broadcast: res.isBroadcast,
					in_block: res.isInBlock,
					retracted: res.isRetracted,
					finalized: res.isFinalized,
					usurped: res.isUsurped,
					dropped: res.isDropped,
					invalid: res.isInvalid,
				};

				let should_check = false;

				if (step.success?.when && step.success.when.status) {
					ctx.debug("CHECKING STATUS", status);

					const expected_status = step.success.when.status;
					// let status_ok = {};

					let each_is_true = true;
					for (const key in expected) {
						// status_ok[key] = (status[key.toLowerCase()] == expected_status[key]);
						if (status[key.toLowerCase()] != expected_status[key]) {
							each_is_true = false;
							break;
						}
					}

					// for (const key in status_ok) {
					// 	if (status_ok[key] != true) {
					// 		each_is_true = false;
					// 	}
					// }

					should_check = each_is_true;
				} else { should_check = true }



				if (should_check && typeof (res) != "function") {
					const result = res["toHuman"] ? res.toHuman() : res;

					ctx.debug("CHECKING RESULT", result);

					for (const key in expected) {
						const success = resolve_key(result, key) == expected[key];
						if (!success) {
							ctx.error("FAIL:", key, ":", resolve_key(result, key), ", but expected:", expected[key]);
							ctx.fail(key + ": " + resolve_key(result, key) + ", but expected: " + expected[key]);
						} else {
							ctx.success();
						}
					}
				}
				return res;
			};
		}
		return f;
	}


	get_nodes_for_step(step) {
		let todo = [];

		if (step.node)
			todo.push(step.node);

		if (step.nodes)
			for (const name of step.nodes) {
				todo.push(name);
			}

		// search
		let found = {};
		for (const name of todo) {
			let norm_name = name.toLowerCase();
			if (this.#nodes[norm_name]) {
				found[name] = this.#nodes[norm_name];
			} else if (this.#nodes[name]) {
				found[name] = this.#nodes[name];
			} else {
				console.warn(`Node ${name} not found.`);
			}
		}

		// console.debug(`using ${Object.keys(found).length} nodes of ${todo.length}`);

		return found;
	}
}



async function make_call_info(step, node, ctx, runner) {
	const provider = new WsProvider("ws://127.0.0.1:" + node.wsPort);
	// --rpc <rpc.json>
	const rpc = {};
	// TODO: types - get from launch configuration
	const types = {};
	const api = await ApiPromise.create({ provider, rpc, types });

	// type: consts, derive, query, rpc, tx
	const [type, section, method] = step.call.method.split(".");

	// TODO: assert:
	const known_types = ["consts", "derive", "query", "rpc", "tx"];
	if (!known_types.includes(type))
		ctx.error(`Expected one of ${known_types.join(", ")}, found ${type}`);

	const fn = api[type][section][method];
	// TODO: assetrinos: https://github.com/polkadot-js/tools/blob/master/packages/api-cli/src/api.ts#L190

	const params = step.call.args;

	ctx.debug("prepare call:", step.call.method);

	return {
		api,
		fn,
		log: (result) => { },
		// ctx.log("[LOG CALL RESULT:",
		// 	stringify({
		// 		[method]: isFunction(result.toHuman)
		// 			? result.toHuman()
		// 			: result
		// 	}, 2),
		// 	"END LOG CALL RESULT]"
		// ),
		check: runner.make_check(step, ctx),
		method,
		section,
		type,
		params,
		//
		sudo: step.call.sudo,
		seed: step.call.signer?.seed,
		cipher: step.call.signer?.cipher // or default sr25519
	};
}



async function query_const(ctx, { fn, log }) {
	log(fn);
	// TODO: implemented me
}


async function send_tx(ctx, { api, fn, log, check, seed, cipher, sudo, params }) {
	const keyring = new Keyring();
	const auth = keyring.createFromUri(seed, {}, is_cipher(cipher) ? cipher : undefined);
	let signable;

	// TODO: get it from params
	const sudoUncheckedWeight = undefined;

	if (sudo || sudoUncheckedWeight) {
		const sudoer = await api.query.sudo.key();

		assert(sudoer.eq(auth.address), 'Supplied seed does not match on-chain sudo key');

		if (sudoUncheckedWeight) {
			signable = api.tx.sudo.sudoUncheckedWeight(fn(...params), sudoUncheckedWeight);
		} else {
			signable = api.tx.sudo.sudo(fn(...params));
		}
	} else {
		signable = fn(...params);
	}

	// TODO: get it from params
	// or if result is not used if `success` check part of step.
	const NO_WAIT = false;

	// return signable.signAndSend(auth, (result) => {
	// 	ctx.log(result);

	// 	if (NO_WAIT || result.isInBlock || result.isFinalized) {
	// 		ctx.debug("TX IN BLOCK!");
	// 		check(result);
	// 	}

	// 	return result;
	// });

	return async () => {
		let ret;
		return signable.signAndSend(auth, (result) => {
			ctx.log(result);

			if (NO_WAIT || result.isInBlock || result.isFinalized) {
				ctx.debug("TX IN BLOCK!");
				// check(result);
				ret = result;
			}

			return result;
		});
		return ret;
	};

}


async function send_call(ctx, { fn, log, check, type, method, params }) {
	let fut = fn(...params);//.then(check);
	// console.debug(f);
	return fut;
	// return fn(...params)
	// 	.then(log)
	// 	.then(() => {
	// 		// process.exit(0);
	// 		console.log("!!! process.exit(0)");
	// 	});
}



const CIPHERS = ["ed25519", "sr25519", "ethereum"];
function is_cipher(s) {
	return CIPHERS.includes((s + "").trim().toLowerCase())
}


exports.TestRunner = TestRunner;
