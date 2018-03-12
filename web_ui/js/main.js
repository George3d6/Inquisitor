/* Pure functions */


// If hearing about System FC fills you with a sense of awe don't go past this point

/* Some god'ol'fashoned global variables and global calls */
const ROUTER = new Navigo(null, true, '#!');


/* Dirty global state using functions */
function toggle_visibility(obj) {
    if('main' in obj) {
        if(obj.main) {
            document.getElementById('main').style.display = 'flex';
        } else {
            document.getElementById('main').style.display = 'none';
        }
    }

    if('header' in obj) {
        if(obj.header) {
            document.getElementById('header').style.display = 'flex';
        } else {
            document.getElementById('header').style.display = 'none';
        }
    }

    if('warning' in obj) {
        if(obj.warning) {
            document.getElementById('warning').style.display = 'block';
        } else {
            document.getElementById('warning').style.display = 'none';
        }
    }
}


async function go_to_view(viw_obj) {
    await place_view_buttons();
    activate_view_button_click();

    toggle_visibility({'main': true, 'header': true, 'warning': false});

    const response = await fetch(`/plugin_data?ts_start=${Math.round(new Date().getTime()/1000) - 4 * 3600}&ts_end=${Math.round(new Date().getTime()/1000)}&level=${viw_obj.level}&name=${viw_obj.name}`);
    const tsv = await response.text();
    const data = tsv.split('\n').map((row) => row.split('\t'));

    const res = display_timeseries(viw_obj.level, viw_obj.name, data, window[`timeseries_map_${viw_obj.level}_${viw_obj.name.replace(/ /gi, '_')}`]);
    close_button_behavior();
    if(res === 'No') {
        toggle_visibility({'warning': true});
        document.getElementById('graph-main').innerHTML = '';
        document.getElementById('warning').innerHTML = `Can't generate graphs for plugin ${viw_obj.name}`;
    }
}


function generate_view_buttons() {
    return __config.plugins.map((plugin) => {
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
    __config.plugins.forEach((plugin) => {
        document.getElementById(plugin.name + plugin.level).addEventListener('click', (e) => {
            ROUTER.navigate(`/view/${encodeURIComponent(plugin.level)}/${encodeURIComponent(plugin.name)}`);
        });
    });
}


async function place_view_buttons() {
    if(__config.plugins === undefined) {
        __config.plugins = [];

        const levels = ['agent', 'receptor'];
        for(let i = 0; i < levels.length; i++) {
            const resp = await fetch(`http://localhost:1834/plugin_list?level=${levels[i]}`);
            const text = await resp.text();
            const plugin_names = text.split('\n');
            for(let n = 0; n < plugin_names.length; n++) {
                console.log(plugin_names[n]);
                __config.plugins.push({
                    'name': plugin_names[n]
                    ,'level': levels[i]
                });
            }
        }
    }
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


ROUTER.notFound((query) => {
    ROUTER.navigate('/404/notfound');
});

ROUTER.on(
    {
        '/': () => {
            toggle_visibility({'main': false, 'header': true})
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
