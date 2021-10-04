const fs = require("fs");
const path = require('path');
const { setTimeout } = require('timers/promises');


exports.setup = setup;

async function setup(cfg) {
	console.log("setting up...");
	// console.log(cfg);

	if (cfg.launch) {
		return {
			runner: "launch",
			result: await setup_launch(cfg.launch)
		};
	}
}

async function setup_launch(cfg) {
	const nodes = read_launch_config(cfg);

	console.log("Running...");

	try {
		const spawn = require('child_process').spawn;
		const p = spawn("polkadot-launch", [cfg.cfg], { cwd: cfg.pwd, detached: false, env: process.env });

		process.on('SIGHUP', () => {
			p.kill();
		});

		if (cfg.success?.wait) {
			p.stdout.setEncoding('utf8');

			let success = false;

			if (cfg.success.wait.stdout) {
				let stdout = "";
				p.stdout.on('data', function (data) {
					data = data.toString();

					// console.log('stdout:', data.includes(cfg.success.wait.stdout), data);

					if (!success) {
						success = data.includes(cfg.success.wait.stdout);
						stdout += data;

						if (success) {
							console.log('✅ setup successed.');
						}
					}
				});
			}

			async function sleep(ms) { return await setTimeout(ms); }

			let counter = 0;
			let timeout = false;

			const time_step = 1000;
			const max_ms = cfg.success.wait.max_secs * 1000;

			while (!success && !timeout) {
				await sleep(time_step);
				counter++;
				const cur_ms = counter * time_step;

				// console.log('waiting... , success:', success, cur_ms, "/", max_ms);

				if (cur_ms >= max_ms) {
					console.log('😖 setup timeout');
					timeout = true;
				}
			}

			console.log("setup complete");

			return {
				process: p,
				success,
				nodes,
			};
		}
	} catch (error) {
		console.error("polkadot-launch error:", error.message);
	}
}


function read_launch_config(cfg) {
	// const launch = require("polkadot-launch");

	function get_parachain_node_name(parachain, node_index) {
		const node = parachain.nodes[node_index];
		return path.parse(parachain.bin).name + "-" + node.wsPort;
	}

	try {
		const data = fs.readFileSync(path.join(cfg.pwd, cfg.cfg), 'utf8');

		const launch_cfg = JSON.parse(data);

		let nodes = [];

		for (const key in launch_cfg.relaychain?.nodes) {
			let o = launch_cfg.relaychain?.nodes[key];
			o.relay = launch_cfg.relaychain.chain;
			nodes.push(o);
		}

		for (const i in launch_cfg.parachains) {
			const para = launch_cfg.parachains[i];

			for (const key in para.nodes) {
				let o = para.nodes[key];
				o.para = para.id;
				o.name = o.name ? o.name : get_parachain_node_name(para, key);
				nodes.push(o);
			}
		}

		let nodes_map = {};
		for (const i in nodes) {
			const node = nodes[i];
			nodes_map[node.name] = node;
		}

		return nodes_map;
	} catch (error) {
		console.error(`Error reading polkadot-launch config file: ${error}`);
	}
}
