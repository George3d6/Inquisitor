/* Pure functions */


// If hearing about System FC fills you with a sense of awe don't go past this point

/* Some god'ol'fashoned global variables */
const ROUTER = new Navigo(null, true, '#!');
const CONFIG = __config;


/* Dirty global state using functions */
function toggle_visibility(obj) {
    if(obj.main) {
        document.getElementById('main').style.display = 'flex';
    } else {
        document.getElementById('main').style.display = 'none';
    }
    if(obj.header) {
        document.getElementById('header').style.display = 'flex';
    } else {
        document.getElementById('header').style.display = 'none';
    }
}


async function go_to_view(viw_obj) {
    toggle_visibility({'main': true, 'header': false});
    const response = await fetch(`/plugin_data?ts_start=0&ts_end=1820197300&level=${viw_obj.level}&name=${viw_obj.name}`);
    const tsv = await response.text();
    const data = tsv.split('\n').map((row) => row.split('\t'));
    console.log(`timeseries_map_${viw_obj.level}_${viw_obj.name.replace(/ /gi, '_')}`);
    display_timeseries(viw_obj.level, viw_obj.name, data, window[`timeseries_map_${viw_obj.level}_${viw_obj.name.replace(/ /gi, '_')}`]);
}


function generate_view_buttons() {
    return CONFIG.plugins.map((plugin) => {
        return `
            <a class="redglow-button header-button" id="${plugin.name + plugin.level}">
                ${plugin.name.toUpperCase()}
                <span class="redglow-button-top"></span>
                <span class="redglow-button-right"></span>
                <span class="redglow-button-bottom"></span>
                <span class="redglow-button-left"></span>
            </a>
        `;
    });
}


function activate_view_button_click() {
    CONFIG.plugins.forEach((plugin) => {
        document.getElementById(plugin.name + plugin.level).addEventListener('click', (e) => {
            ROUTER.navigate(`/view/${encodeURIComponent(plugin.level)}/${encodeURIComponent(plugin.name)}`);
        });
    });
}


function place_view_buttons() {
    document.getElementById('header').innerHTML = generate_view_buttons().join('\n');
}


function display_not_found() {
    toggle_visibility({'main': true, 'header': false});
    const main = document.getElementById('main');
    main.innerHTML = '';
    main.style.background = 'url(/img/notfound.jpg) no-repeat top center fixed';
    main.style.backgroundSize = 'contain';
    main.style.backgroundColor = 'rgb(0,6,22)';
}


ROUTER.notFound(function (query) {
    ROUTER.navigate('/404/notfound')
});

ROUTER.on(
    {
        '/': () => {
            toggle_visibility({'main': false, 'header': true})
            place_view_buttons();
            activate_view_button_click();
        }
        ,'/view/:level/:plugin_name': (params) => {
            const plugin_name = decodeURIComponent();
            go_to_view({'name': params.plugin_name, 'level': params.level});
        }
        ,'/404/notfound': (params) => {
            display_not_found();
        }
    }
).resolve();
