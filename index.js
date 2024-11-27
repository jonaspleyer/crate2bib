import init, { create_bib_string } from './pkg/crate2bib.js';

async function run() {
    await init();
}

globalThis.create_bib_string = function(x) {
    create_bib_string(x).then(
        function(value) {value},
        function(error) {console.log(error);}
    )
};

run();
