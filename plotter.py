import json
import matplotlib.pyplot as plt
import os
import math
import numpy as np
from tabulate import tabulate
from collections import defaultdict
from matplotlib.lines import Line2D
import re

def create_folders(base_path, folder_list):
    for folder in folder_list:
        path_to = "{}/{}".format(base_path, folder)
        if not os.path.exists(path_to):
            os.makedirs(path_to)

def export_legend(legend, filename):
    fig  = legend.figure
    fig.canvas.draw()
    bbox  = legend.get_window_extent().transformed(fig.dpi_scale_trans.inverted())
    fig.savefig(filename, dpi="figure", bbox_inches=bbox)

def pareto_frontier(points):
    sorted_points = sorted(points, key=lambda p: (p[0], p[1]))
    pareto_front = []
    min_y = float('inf')
    for x, y in sorted_points:
        if y < min_y:
            pareto_front.append((x, y))
            min_y = y
    return pareto_front

def extract_idx_root(idx):
    root = re.sub(r" ?\d+$", "", idx)
    root = root.replace("RMI-compact", "RMI")
    root = root.replace("RMI-large", "RMI")
    return root

# Create necessary folders for output
folders = ["buildtime", "buildtime/zoom", "index", "index/pareto", "index/space", "legends", "spacetime", 
           "tables", "vector", "vector/pareto", "vector/scan", "vector/space"]
create_folders("output/plots", folders)

# Define datasets and other configurations
tool_performance = {}
space_occupancy = {}

bottom_plots = ["wiki_ts_200M_uint32", "zipf_uint32", "wiki_ts_200M_uint64"]
pareto_dataset = ["books_200M_uint32", "zipf_uint32", "lognormal_uint32", "osm_cellids_800M_uint64"]

list_of_markers = [m for m in Line2D.markers.keys() if m not in [",", "None", None, " ", ""]] 
list_of_colors = ["blue", "lime", "yellow", "grey", "orange", "green", "purple", "navy", "lightsalmon", "deepskyblue", "olive", 
                  "gold", "lightcoral", "magenta", "yellowgreen", "brown", "violet", "seagreen", "khaki", "chocolate"] 

color_map = {"std::vector": "red"}
marker_map = {}

datasets = {
    "zipf_uint32": "zipf", 
    "books_200M_uint32": "books", 
    "companynet_uint32": "companynet", 
    "fb_200M_uint64": "fb64", 
    "lognormal_uint32": "lognormal", 
    "normal_uint32": "normal", 
    "wiki_ts_200M_uint32": "wiki", 
    "books_800M_uint64": "books64", 
    "exponential_uint32": "exponential", 
    "friendster_50M_uint32": "friendster",
    "osm_cellids_800M_uint64": "osm64", 
    "wiki_ts_200M_uint64": "wiki64"
}

# Read memory usage data
curr_dataset = ""
memory_usage = defaultdict(list)
memory_std_vec = {}
with open("output/memory_footprint.txt") as memory_usage_f:
    for line in memory_usage_f.readlines():
        if not line: 
            continue
        if " - " in line: 
            index_, mb = line.split(" - ")
            if index_ == "std::vector":
                memory_std_vec[curr_dataset] = int(mb)
            else:
                memory_usage[index_].append((int(mb), curr_dataset))    
        else:
            curr_dataset = line.strip()

# Write memory ratios to file
with open("output/memory_ratios.txt", 'w') as recap_memory_f:
    for algo, list_res in memory_usage.items():
        max_ratio, min_ratio = 0, 500000
        max_datas, min_datas = "", ""
        for mb, dataset in list_res:
            curr_ratio = int(mb) / memory_std_vec[dataset]
            if curr_ratio > max_ratio:
                max_ratio = curr_ratio
                max_datas = dataset
            if curr_ratio < min_ratio:
                min_ratio = curr_ratio
                min_datas = dataset
        recap_memory_f.write("{}: MAX: {:.2f}x ({}) - MIN: {:.2f}x ({})\n".format(algo, max_ratio, max_datas, min_ratio, min_datas))

# Read performance data from JSON files
marker_idx, color_idx = 0, 0
all_indices = set()
for experiment in ["existing", "missing", "buildtime", "scan"]:
    path_to = "output/"+experiment + ".json"
    with open(path_to, 'r') as json_file:
        data = json.load(json_file)['benchmarks']
    for entry in data:
        full_bm_str = entry["name"].split("/")[0]
        real_time_ns = entry["real_time"]
        dataset = [a for a in datasets.keys() if a in full_bm_str][0]
        full_bm_str = full_bm_str.replace(dataset + "_", '')
        if experiment == "scan":
            num_scans = int(full_bm_str.split("_")[0])
            index_ = full_bm_str.split("_")[1]
            tool_performance[experiment + "_" + dataset + "_" + str(num_scans) + "_" + index_] = (real_time_ns / (num_scans*100000))
        else:
            index_ = full_bm_str.split("_")[0]
            tool_performance[experiment + "_" + dataset + "_" + index_] = real_time_ns / 1000000
        if index_ not in all_indices:
            all_indices.add(index_)
            root_string = extract_idx_root(index_)
            curr_color = [v for k, v in color_map.items() if extract_idx_root(k) == root_string]
            if curr_color:
                color_map[index_] = curr_color[0]
            else:
                color_map[index_] = list_of_colors[color_idx]
                color_idx = color_idx + 1
            marker_map[index_] = list_of_markers[marker_idx]
            marker_idx = marker_idx + 1

# understanding if the index is a compressed index or not (if the index_ops.hpp file has the access method)
is_compressed_idx_map = {k: False for k in all_indices}
for filename in os.listdir("include/index_ops"):
    with open("include/index_ops/" + filename, 'r') as index_ops_file:
        is_compressed_idx = False
        read_next_line = False
        for line in index_ops_file.readlines(): 
            if read_next_line:
                read_next_line = False
                line = line.strip()
                line = line.replace("return ", "")
                match = re.findall('"([^"]*)"', line)[0]
                to_update = [k for k in is_compressed_idx_map.keys() if match in k]
            if "T access" in line:
                is_compressed_idx = True
            if "string to_string" in line:
                read_next_line = True
        for update_to_make in to_update:
            is_compressed_idx_map[update_to_make] = is_compressed_idx

# Read index sizes
with open("output/index_sizes.txt", 'r') as index_size_file:
    for line in index_size_file.readlines():
        dataset, index_, bytes_ = line.split(" - ")
        bytes_ = bytes_.replace("\n", '')
        space_occupancy[dataset.split(".")[0] + "_" + index_] = math.ceil(float(bytes_))

# Generate tables for each dataset
header_column = ['index', 'lookup existing (ns)', 'lookup missing (ns)', 'space (MB)', 'build']
header_column_with_scan = ['index', 'lookup existing (ns)', 'lookup missing (ns)', 'space (MB)', 'build', "Scan 10", "Scan 100", "Scan 1K", "Scan 10K"]
for dataset in datasets.keys():
    data_to_table = []
    all_experiments_with_dataset = [a for a in tool_performance.keys() if dataset in a] 
    indices_for_this_dataset = sorted([idx_ for idx_ in all_indices if any([idx_ in exp for exp in all_experiments_with_dataset])])
    scan_present = any(["scan_" in exp for exp in all_experiments_with_dataset])
    for idx_ in indices_for_this_dataset:
        ns_present = tool_performance.get("existing_{}_{}".format(dataset, idx_), "-")
        ns_missing = tool_performance.get("missing_{}_{}".format(dataset, idx_), "-")
        ns_build = tool_performance.get("buildtime_{}_{}".format(dataset, idx_), "-")
        occupancy = space_occupancy.get("{}_{}".format(dataset, idx_), "-")
        scan_perf = []
        if scan_present:
            for scan_range in [10, 100, 1000, 10000]:
                scan_perf.append(tool_performance.get("existing_{}_{}_{}".format(dataset, scan_range, idx_), "-"))
            data_to_table.append((idx_, ns_present, ns_missing, f'{occupancy:,}', ns_build, scan_perf[0], scan_perf[1], scan_perf[2], scan_perf[3]))
        else: 
            data_to_table.append((idx_, ns_present, ns_missing, f'{occupancy:,}', ns_build))
    with open('output/plots/tables/{}.tex'.format(dataset), 'w') as f:
        if scan_present:
            f.write(tabulate(data_to_table, tablefmt="latex", headers=header_column_with_scan))
        else:
            f.write(tabulate(data_to_table, tablefmt="latex", headers=header_column))
    
    perf = {"buildtime": [], "existing": [], "missing": [], "indices": [], "scan10": [],"scan100": [], "scan1000": [], "scan10000": [], "scan_indices": set()}
    for idx_ in indices_for_this_dataset:
        for experiment in ["buildtime", "existing", "missing"]:
            key_ = "{}_{}_{}".format(experiment, dataset, idx_)
            perf[experiment].append(tool_performance[key_])
        perf["indices"].append(idx_)
        perf["scan_present"] = scan_present
        if scan_present:
            for scan_range in [10, 100, 1000, 10000]:
                key_ = "scan_{}_{}_{}".format(dataset, scan_range, idx_)
                if key_ in tool_performance: 
                    perf["scan_indices"].add(idx_)
                    perf["scan{}".format(scan_range)].append(tool_performance[key_])
    # buildtime
    fig, ax = plt.subplots()
    zipped_list = [(idx_, buildtime) for idx_, buildtime in zip(perf["indices"], perf["buildtime"]) if idx_ != "std::vector"]
    ax.bar([a for a, _ in zipped_list], [a for _, a in zipped_list], color = [color_map[a] for a, _ in zipped_list], edgecolor = "black")
    if dataset not in bottom_plots:
        ax.get_xaxis().set_ticks([])
    ax.set_ylabel('ms to build', fontsize=13)
    plt.yticks(ticks=plt.yticks()[0], labels=plt.yticks()[0].astype(int))
    plt.xticks(rotation=45, ha="right")
    plt.savefig("output/plots/buildtime/{}.png".format(dataset), bbox_inches='tight')
    plt.clf()
    # zoom
    fig, ax = plt.subplots()
    avg_ = sum(perf["buildtime"]) / len(perf["buildtime"])
    zipped_list = [(idx_, buildtime) for idx_, buildtime in zip(perf["indices"], perf["buildtime"]) if idx_ != "std::vector" and buildtime <= 2*avg_]
    ax.bar([a for a, _ in zipped_list], [a for _, a in zipped_list], color = [color_map[a] for a, _ in zipped_list], edgecolor = "black")
    if dataset not in bottom_plots:
        ax.get_xaxis().set_ticks([])
    ax.set_ylabel('ms to build', fontsize=13)
    plt.yticks(ticks=plt.yticks()[0], labels=plt.yticks()[0].astype(int))
    plt.xticks(rotation=45, ha="right")
    plt.savefig("output/plots/buildtime/zoom/{}.png".format(dataset), bbox_inches='tight')
    plt.clf()
    # index and compressed index plots
    spacetime_indices = []
    for plot_compressed_indices in [True, False]:
        fig, ax = plt.subplots()
        width = 0.4
        idx_of_correct_indices = [i for i, idx_ in enumerate(perf["indices"]) if is_compressed_idx_map[idx_] == plot_compressed_indices]
        indices = [perf["indices"][i] for i in idx_of_correct_indices]
        spacetime_indices.extend(indices)
        if plot_compressed_indices: 
            spacetime_indices.remove("std::vector")
            spacetime_indices.append("std::vector") # so it gets at the end
        values = [perf["existing"][i] for i in idx_of_correct_indices]
        values_missing = [perf["missing"][i] for i in idx_of_correct_indices]
        if not plot_compressed_indices:
            indices.append("std::vector")
            values.append(tool_performance["existing_"+dataset+"_std::vector"])
            values_missing.append(tool_performance["missing_"+dataset+"_std::vector"])
        colors = [color_map[idx_] for idx_ in indices]
        ind = np.arange(len(indices))
        ax.bar(ind, values, width, color = colors, edgecolor = "black")
        ax.bar(ind + width, values_missing, width, color = colors, edgecolor = "black")
        ax.set_ylabel('ns per item', fontsize=13)
        plt.xticks(ind + width / 2, indices)
        plt.xticks(rotation=45, ha="right")
        root_to_file = "vector" if plot_compressed_indices else "index"
        plt.savefig("output/plots/{}/{}.png".format(root_to_file, dataset), bbox_inches='tight')
        plt.clf()

        # space
        fig, ax = plt.subplots()
        ind = np.arange(len(indices))
        width = 0.6  
        space_occup = [space_occupancy[dataset + "_" + idx_] for idx_ in indices]
        ax.bar(ind, space_occup, width, color = colors, edgecolor = "black")
        ax.set_ylabel('Size (in MB)', fontsize=13)
        plt.xticks(ind, indices)
        plt.xticks(rotation=45, ha="right")
        root_to_file = "vector" if plot_compressed_indices else "index"
        plt.savefig("output/plots/{}/space/{}.png".format(root_to_file, dataset), bbox_inches='tight')
        plt.clf()

        # scan plots
        if plot_compressed_indices and perf["scan_present"]:
            fig, ax = plt.subplots()
            width = 0.2
            ind = np.arange(len(perf["scan_indices"]))  
            colors_scan = [color_map[idx_] for idx_ in perf["scan_indices"]]
            for i, scan_type in enumerate(["scan10", "scan100", "scan1000", "scan10000"]):
                ax.bar(ind + (width*i), perf[scan_type], width, color = colors_scan, edgecolor = "black")
            ax.set_ylabel('ns per item', fontsize=13)
            plt.xticks(ind + width + width/2, perf["scan_indices"])
            plt.xticks(rotation=45, ha="right")
            root_to_file = "vector/scan"
            plt.savefig("output/plots/{}/{}.png".format(root_to_file, dataset), bbox_inches='tight')
            plt.clf()
            handles = [plt.Rectangle((0,0),1,1, color=col) for col in colors_scan]
            for handle in handles: 
                handle.set_edgecolor('black')
            n_columns = math.ceil(len(perf["scan_indices"])/3)
            if not dataset.endswith("64"):
                legend = plt.legend(handles, perf["scan_indices"], loc='upper center', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=n_columns)
                export_legend(legend, "output/plots/legends/scan.png")
            plt.clf()

        # pareto plots
        if dataset in pareto_dataset:
            fig, ax = plt.subplots()
            if not plot_compressed_indices:
                algorithms_filtered_idx = [i for i, a in enumerate(indices) if a != "std::vector" and space_occupancy[dataset+"_"+a] >= 1e+8]
                algorithms_filtered = [perf["indices"][i] for i in algorithms_filtered_idx]
                values_filtered = [perf["existing"][i] for i in algorithms_filtered_idx]
                markers__ = [marker_map[a] for a in algorithms_filtered]
                colors__ = [color_map[a] for a in algorithms_filtered]
            else:
                algorithms_filtered = indices
                values_filtered = values
                markers__ = [marker_map[a] for a in algorithms_filtered]
                colors__ = [color_map[a] for a in algorithms_filtered]
            to_pareto = []
            for i, algo in enumerate(algorithms_filtered):
                ax.plot(space_occupancy[dataset + "_" + algo], values_filtered[i], marker=markers__[i], color=colors__[i], label=algo)
                to_pareto.append((space_occupancy[dataset + "_" + algo], values_filtered[i]))
            pareto = pareto_frontier(to_pareto)
            x_pareto, y_pareto = zip(*pareto)
            plt.plot(x_pareto, y_pareto, color='black', linestyle='-')
            root_to_file = "vector" if plot_compressed_indices else "index"
            ax.set_ylabel('ns per item', fontsize=13)
            ax.set_xlabel('Size (in MB)', fontsize=13)
            plt.savefig("output/plots/{}/pareto/{}.png".format(root_to_file, dataset), bbox_inches='tight')
            if dataset == "books_200M_uint32":
                ax.set_xlabel('')
                if not plot_compressed_indices:
                    legend = plt.legend(loc='upper right', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=n_columns)
                    export_legend(legend, "output/plots/legends/pareto_index.png")
                else:
                    n_columns = 4
                    legend = plt.legend(loc='upper right', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=n_columns)
                    export_legend(legend, "output/plots/legends/pareto_vector.png")
            plt.clf()

    # computing optimal plots for spacetime 
    filtered_spacetime_indices = {}
    for a in spacetime_indices:
        root_str = extract_idx_root(a)
        if root_str not in filtered_spacetime_indices:
            options_indices = [x for x in spacetime_indices if extract_idx_root(x) == root_str]
            list_of_options = [(x, tool_performance["existing_{}_{}".format(dataset, x)],space_occupancy[dataset + "_" + x]) for x in options_indices]
            max_space = max(ds[2] for ds in list_of_options)
            max_time = max(ds[1] for ds in list_of_options)
            normalized_scores = []
            for idx_, space_occup, query_time in list_of_options:
                norm_space = space_occup / max_space
                norm_time = query_time / max_time
                score = norm_space + norm_time  # Lower is better
                normalized_scores.append((score, idx_))
            best_idx = min(normalized_scores, key=lambda x: x[0])[1]
            filtered_spacetime_indices[root_str] = {
                "opt": best_idx,
                "root_str": root_str,
                "existing": tool_performance["existing_{}_{}".format(dataset, best_idx)],
                "space": space_occupancy[dataset + "_" + best_idx],
            }
    # spacetime plots
    i = 0
    spacetime_plot_positions = []
    algorithms_to_plot = [a["opt"] for a in filtered_spacetime_indices.values()]
    ns_to_plot = [a["existing"] for a in filtered_spacetime_indices.values()]
    root_str_list = [a["root_str"] for a in filtered_spacetime_indices.values()]
    space_to_plot = [a["space"] for a in filtered_spacetime_indices.values()]
    spacetime_plot_colors = [color_map[a] for a in algorithms_to_plot]
    for a in filtered_spacetime_indices.values():
        if a["opt"] == "std::vector":
            spacetime_plot_positions.append(i + 1)
            i = i + 3
        else:
            spacetime_plot_positions.append(i)
            i = i + 1
    ax1 = plt.subplot(2,1,1)
    ax1.tick_params(left = False, right = False, labelleft = False, labelbottom = False, bottom = False) 
    ax1.bar(spacetime_plot_positions, ns_to_plot, color = spacetime_plot_colors, edgecolor = "black")
    ax1.set_ylabel('lookup in ns')
    ax1.set_ylim(0, max(ns_to_plot) * 1.4) 
    for j, i in enumerate(spacetime_plot_positions):
        ax1.text(i, ns_to_plot[j]+(max(ns_to_plot) / 15), ns_to_plot[j], ha = 'center', rotation=45)
    ax2 = plt.subplot(2,1,2)
    ax2.tick_params(left = False, right = False, labelleft = False, labelbottom = False, bottom = False) 
    ax2.bar(spacetime_plot_positions, space_to_plot, color = spacetime_plot_colors, edgecolor = "black")
    ax2.set_ylabel('size in MB')
    ax2.set_ylim(0, max(space_to_plot) * 1.2) 
    for j, i in enumerate(spacetime_plot_positions):
        ax2.text(i, space_to_plot[j]+(max(space_to_plot) / 6), space_to_plot[j], ha = 'center',rotation=45)
    ax2.invert_yaxis()
    plt.savefig("output/plots/spacetime/{}.png".format(dataset), bbox_inches='tight')
    # legends
    plt.clf()
    handles = [plt.Rectangle((0,0),1,1, color=col) for col in spacetime_plot_colors]
    zipped_list = [(a, b) for a, b in zip(handles, algorithms_to_plot)]
    idx_of_stdvector = algorithms_to_plot.index("std::vector")
    for handle in handles: 
        handle.set_edgecolor('black')
    n_columns = math.ceil(len(root_str_list)/3)
    vect_new_hand = [handles[i] for i in range(len(handles)) if i <= idx_of_stdvector]
    vect_new_algos = [algorithms_to_plot[i] for i in range(len(handles)) if i <= idx_of_stdvector]
    ind_new_hand = [handles[i] for i in range(len(handles)) if i >= idx_of_stdvector]
    ind_new_algos = [algorithms_to_plot[i] for i in range(len(handles)) if i >= idx_of_stdvector]
    if dataset == "books_800M_uint64":
        legend = plt.legend(handles, algorithms_to_plot, loc='upper center', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=n_columns)
        export_legend(legend, "output/plots/legends/64_no_label.png")
    if dataset == "books_200M_uint32":
        legend = plt.legend(handles, algorithms_to_plot, loc='upper center', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=n_columns)
        export_legend(legend, "output/plots/legends/no_label.png")
    if dataset == "wiki_ts_200M_uint64":
        legend_vec = plt.legend(vect_new_hand, vect_new_algos, loc='upper center', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=math.ceil(len(vect_new_algos)/2))
        export_legend(legend_vec, "output/plots/legends/vec64.png")
    if dataset == "wiki_ts_200M_uint32":    
        legend_vec = plt.legend(vect_new_hand, vect_new_algos, loc='upper center', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=math.ceil(len(vect_new_algos)/2))
        export_legend(legend_vec, "output/plots/legends/vec.png")
    if dataset == "companynet_uint32":
        legend_ind = plt.legend(ind_new_hand, ind_new_algos, loc='upper center', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=math.ceil(len(ind_new_algos)/2))
        export_legend(legend_ind, "output/plots/legends/index.png")
    if dataset == "osm_cellids_800M_uint64":
        legend_ind = plt.legend(ind_new_hand, ind_new_algos, loc='upper center', bbox_to_anchor=(0.5, -0.05), framealpha=1, frameon=False, shadow=True, ncol=math.ceil(len(ind_new_algos)/2))
        export_legend(legend_ind, "output/plots/legends/index64.png")
    plt.clf()