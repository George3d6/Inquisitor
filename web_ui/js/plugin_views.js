function display_timeseries(plugin_level, plugin_name, data, generate_timeseries_map) {

    if(generate_timeseries_map === undefined) {
        console.log("No function to plot this view !");
        return "No";
    }
    const timeseries_map = generate_timeseries_map(data);

    document.getElementById('graph-holder').innerHTML = '';
    for (let k in timeseries_map) {
        const points = timeseries_map[k].points;
        const metadata = timeseries_map[k].metadata;
        document.getElementById('graph-holder').innerHTML += `<svg id="graph-${k.replace(/ /g, "-")}" class="graph" width="800" height="900"></svg>`;

        const svg = d3.select(`#graph-${k.replace(/ /g, "-")}`);
        const margin = {top: 50, right: 20, bottom: 30, left: 50};
        const width = +svg.attr('width') - margin.left - margin.right;
        const height = +svg.attr('height') - margin.top - margin.bottom;
        const g = svg.append('g').attr('transform', 'translate(' + margin.left + ',' + margin.top + ')');

        svg.append("text")
        .attr("x", (width / 2))
        .attr("y", 24)
        .attr("text-anchor", "middle")
        .style("font-size", "24px")
        .text(`${plugin_name} - ${k}`);

        const x = d3.scaleTime().rangeRound([0, width]);

        const y = d3.scaleLinear().rangeRound([height, 0]);

        const line = d3.line().x(function(d) { return x(d.ts); }).y(function(d) { return y(d.val); });

        x.domain(d3.extent(points, (point) => {  return parseFloat(point.ts) }));

        if('ymin' in metadata && 'ymax' in metadata) {
            y.domain([metadata.ymin, metadata.ymax]);
        } else {
            y.domain(d3.extent(points, (point) => {  return parseFloat(point.val) }));
        }



        g.append('g')
        .attr('transform', 'translate(0,' + height + ')')
        .call(d3.axisBottom(x))
        .select('.domain')
        .remove();

        g.append('g')
            .call(d3.axisLeft(y))
            .append('text')
            .attr('fill', 'rgb(16,16,16)')
            .attr('transform', 'rotate(-90)')
            .attr('y', 6)
            .attr('dy', '0.71em')
            .attr('text-anchor', 'end')
            .attr('font-size', '14px')
            .text(metadata.ylabel);

        g.append('path')
            .datum(points)
            .attr('fill', 'none')
            .attr('stroke', ' rgb(146, 42, 62)')
            .attr("stroke-width", 6)
            .attr('stroke-linejoin', 'round')
            .attr('stroke-linecap', 'round')
            .attr('stroke-width', 1.5)
            .attr('d', line)
            .attr('font-size', '14px')
            .text('Time');
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
                ,'ylable': 'Time difference between agent and receptor (seconds)'
            };
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

            if (!(`${sender_name} used memory` in timeseries_map)) {
                timeseries_map[`${sender_name} used memory`] = {'points': []};
            };

            timeseries_map[`${sender_name} used memory`].points.push({'ts': row[2], 'val': system_state['memory_map']['used_memory']})

            if(!('metadata' in timeseries_map[`${sender_name} used memory`])) {
                timeseries_map[`${sender_name} used memory`].metadata = {
                    'ymax': system_state['memory_map']['total_memory']
                    ,'ymin': 0
                    ,'ylable': 'Memory usage (bytes)'
                };
            }


    });
    return timeseries_map;
}
