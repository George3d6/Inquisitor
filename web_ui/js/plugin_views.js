function display_timeseries(plugin_level, plugin_name, data, generate_timeseries_map) {

    if(generate_timeseries_map === undefined) {
        return "No";
    }
    const timeseries_map = generate_timeseries_map(data);

    document.getElementById('graph-main').innerHTML = '';
    for (let k in timeseries_map) {
        let points = timeseries_map[k].points;
        const metadata = timeseries_map[k].metadata;
        document.getElementById('graph-main').innerHTML += `<div id="graph-holder-${k.replace(/ /g, "-")}" class="graph_holder">
                                                                <div id="graph-header-${k.replace(/ /g, "-")}" class="graph_header">
                                                                <div id="graph-button-${k.replace(/ /g, "-")}" class="open open-close-button"></div>
                                                                <p class="graph_header_text">${plugin_name} - ${k}</p>
                                                                </div>
                                                                <svg id="graph-${k.replace(/ /g, "-")}" class="graph"  height="380" width="420"></svg>
                                                             </div>`;


        const svg = d3.select(`#graph-${k.replace(/ /g, "-")}`);
        const margin = {top: 50, right: 20, bottom: 30, left: 40};
        const width = +svg.attr('width') - margin.left - margin.right;
        const height = +svg.attr('height') - margin.top - margin.bottom;

        const parse_time = d3.timeParse("%d-%b-%y");

        points = points.map((point) => {
            point.ts = (point.ts * 1000);
            return point;
        });

        // Set the ranges and define the line
        const x = d3.scaleTime().rangeRound([0, width]);
        const y = d3.scaleLinear().rangeRound([height, 0]);
        const line = d3.line().x(function(d) { return x( d.ts ); }).y(function(d) { return y(d.val); });

        x.domain(d3.extent(points, (p) => { return p.ts; }));

        if('ymin' in metadata && 'ymax' in metadata) {
            y.domain([metadata.ymin, metadata.ymax]);
        } else {
            y.domain([0, d3.max(points, (p) =>  { return parseFloat(p.val); })]);
        }


        const g = svg.append('g').attr("class", "axisY").attr('transform', 'translate(' + margin.left + ',' + margin.top + ')');
        svg.append('g').attr("transform", "translate(35," + (50) + ")").attr('class', 'axis').call(d3.axisLeft(y));
        svg.append('g').attr("transform", "translate(35," + (height + 50) + ")").attr('class', 'axis').call(d3.axisBottom(x));

        g.append('path')
            .data([points])
            .attr('fill', 'none')
            .attr('stroke', 'rgb(250,250,250)')
            .attr("stroke-width", 3)
            .attr('stroke-linejoin', 'round')
            .attr('stroke-linecap', 'round')
            .attr('d', line)
            .attr('text-anchor', 'middle');

        svg.append("text")
        .attr("transform",  "translate(" + (width/2) + " ," + (height + margin.top + 40) + ")")
        .style('fill', 'rgb(250,250,250)')
        .style("text-anchor", "middle")
        .text("Date");

        svg.append("text")
        .attr("transform", "rotate(-90)")
        .attr("y", 0 - margin.left - 10)
        .attr("x", 0 - (height / 2) - 35)
        .attr("dy", "1em")
        .style('fill', 'rgb(250,250,250)')
        .style("text-anchor", "middle")
        .text(metadata.ylabel);

    }
    return "Ok";
}


function timeseries_map_receptor_Sync_check(data) {
    const timeseries_map = [];
    data.forEach((row) => {
        const message_obj = JSON.parse(row[0]);
        const sender_name = Object.keys(message_obj)[0];
        if(sender_name === undefined) {
            return;
        }
        if (!(sender_name in timeseries_map)) {
            timeseries_map[sender_name] = {'points': []};
        };
        timeseries_map[sender_name].points.push({'ts': row[1], 'val': message_obj[sender_name]})

        if(!('metadata' in timeseries_map[sender_name])) {
            timeseries_map[sender_name].metadata = {
                'ymax': 30
                ,'ymin': -30
                ,'ylabel': 'Time difference between agent and receptor (seconds)'
            };
        }
    });
    return timeseries_map;
}


function timeseries_map_agent_Process_counter(data) {
    const timeseries_map = [];
    data.forEach((row) => {
        const obj = JSON.parse(row[1]);
        const sender_name = row[0];
        const ts = row[2];

        for(let process_name in obj) {
            if (!(`${sender_name} - ${process_name}` in timeseries_map)) {
                timeseries_map[`${sender_name} - ${process_name}`] = {'points': []};
            };
            timeseries_map[`${sender_name} - ${process_name}`].points.push({'ts': row[2], 'val': obj[process_name]});
            timeseries_map[`${sender_name} - ${process_name}`].metadata = {'ylabel': 'Nr of processes running'};
        }
    });

    return timeseries_map;
}


function timeseries_map_agent_System_monitor(data) {
/*
{"fs_state":[{"mount_point":"/","available_space":"47677444096","total_space":"222003179520"},{"total_space":"535805952","available_space":"493494272","mount_point":"/boot"},
{"mount_point":"/var/lib/docker/plugins","available_space":"47677444096","total_space":"222003179520"},
{"mount_point":"/var/lib/docker/overlay2","available_space":"47677444096","total_space":"222003179520"}],
"memory_map":{"total_memory":"8075164","used_memory":"4604136"},"swap_map":{"total_swap":"12582908","used_swap":"512"},"processor_map":{"total_usage":0.4169175},"network_map":{"out":"0","in":"0"}}
 */
    const timeseries_map = [];
    data.forEach((row) => {
        const system_state = JSON.parse(row[1]);
        const sender_name = row[0];
        const ts = row[2];

        /* Memory */

        if (!(`${sender_name} used memory` in timeseries_map)) {
            timeseries_map[`${sender_name} used memory`] = {'points': []};
        };

        timeseries_map[`${sender_name} used memory`].points.push({'ts': row[2], 'val': system_state['memory_map']['used_memory']})

        if(!('metadata' in timeseries_map[`${sender_name} used memory`])) {
            timeseries_map[`${sender_name} used memory`].metadata = {
                'ymax': system_state['memory_map']['total_memory']
                ,'ymin': 0
                ,'ylabel': 'Memory usage (bytes)'
            };
        }

        /* CPU */

        if (!(`${sender_name} used processor_map` in timeseries_map)) {
            timeseries_map[`${sender_name} used processor_map`] = {'points': []};
        };

        timeseries_map[`${sender_name} used processor_map`].points.push({'ts': row[2], 'val': system_state['processor_map']['total_usage']})

        if(!('metadata' in timeseries_map[`${sender_name} used processor_map`])) {
            timeseries_map[`${sender_name} used processor_map`].metadata = {
                'ymax': 1
                ,'ymin': 0
                ,'ylabel': 'CPU usage (%)'
            };
        }

    });
    return timeseries_map;
}
