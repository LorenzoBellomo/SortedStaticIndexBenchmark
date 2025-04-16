#!/usr/bin/env python3
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
from pathlib import Path
import tabulate
from matplotlib.ticker import LogLocator
from matplotlib.colors import to_rgba
import re
import argparse
import sys

# import mpld3
# import plotly.io as pio
# import plotly
# import plotly.tools as tls

store_dir = "plots"
palette = None
dashes = {"": ""}
human = ""
release = ""
out_format = "png"
input_file_prefix = "results/results"


def caches():
    sizes = []
    # Note: index1 for me is the L1 instruction cache.
    # Note: All read strings are eg 32K.
    for i, name in [(0, "L1"), (2, "L2"), (3, "L3")]:
        t = Path(f"/sys/devices/system/cpu/cpu0/cache/index{i}/size").read_text()
        sizes.append((int(t[:-2]) * 1024, name))
    return sizes


(l1_size, _), (l2_size, _), (l3_size, _) = caches()


def plot(
    experiment_name,
    title,
    data,
    names,
    # Previous results are dimmed,
    # but the most recent one is bolded.
    keep,
    new_best=None,
    skip=0,
    ymax=None,
    latency=False,
    style="Style",
    size=False,
    highlight=1,
    spread=False,
    # either false or an array specifying how many threads each name has
    # to allow for plots with mixed threading
    threads=[],
):
    print("plotting", experiment_name, "\n", names)
    # Create a figure
    fig, ax = plt.subplots(figsize=(11, 8))
    ax.set_title(title)
    ax.set_xlabel("Input size (bytes)")
    if latency:
        ax.set_ylabel("Latency (ns)")
    else:
        ax.set_ylabel("Inverse throughput (ns)")

    global dashes
    # for s in data.Style.unique():
    # assert s in dashes

    def threads_names_map(thr, name):
        return f"T{thr}: {name}"

    data = data.copy()
    if not threads:
        data = data[data["name"].isin(names)]
    else:
        zipped_all = zip(data["threads"], data["name"])
        zipped_selected = zip(threads, names)
        data["name"] = list(map(lambda thr_name: threads_names_map(thr_name[0], thr_name[1]), zipped_all))
        names = list(map(lambda thr_name: threads_names_map(thr_name[0], thr_name[1]), zipped_selected))
        data = data[data["name"].isin(names)]
    new_best = (
        [] if new_best is False else new_best or (names[-highlight:] if names else [])
    )

    def type(name):
        if keep and name == keep[-1]:
            return "best"
        if keep and highlight == 2 and name == keep[-2]:
            return "best"
        if name in new_best:
            return "new_best"
        if name in keep:
            return "old"
        return "new"

    data["type"] = data.name.apply(type)

    alpha_map = {"old": 0.3, "new": 1, "best": 1, "new_best": 1}

    if not threads:
        my_palette = {name: to_rgba(palette[name], alpha_map[type(name)]) for name in names}
        sns.lineplot(
            x="sz",
            y="latency",
            hue=data[
                "name"
                # "Color" if len(data.batchsize.unique()) == 1 else "display_name"
            ].tolist(),
            style=style if data.Style.unique().tolist() != [""] else None,
            data=data,
            legend="auto",
            size="type",
            sizes={"old": 0.5, "new": 1, "best": 2, "new_best": 1.5},
            palette=my_palette,
            errorbar=("pi", 50) if spread else None,
            estimator="median",
        )
    else:
        sns.lineplot(
            x="sz",
            y="latency",
            hue=data[
                "name"
                # "Color" if len(data.batchsize.unique()) == 1 else "display_name"
            ].tolist(),
            style=style if data.Style.unique().tolist() != [""] else None,
            data=data,
            legend="auto",
            size="type",
            sizes={"old": 0.5, "new": 1, "best": 2, "new_best": 1.5},
            # palette=my_palette,
            errorbar=("pi", 50) if spread else None,
            estimator="median",
        )

    # Plot index size with a separate y-scale.
    if size:
        size_ax = ax.twinx()
        sns.lineplot(
            x="sz",
            y="capped_overhead",
            hue=data["name"].tolist(),
            # style=style,
            # linestyle="dotted",
            style=style if data.Style.unique().tolist() != [""] else None,
            size="type",
            sizes={"old": 0.5, "new": 1, "best": 2, "new_best": 1.5},
            data=data,
            legend=None,
            palette=palette,
            ax=size_ax,
            errorbar=("pi", 50) if spread else None,
            estimator="median",
        )

        size_ax.set_ylim(0, 7)
        size_ax.grid(True, alpha=0.4, ls="dotted", lw=1)
        size_ax.set_ylabel("Size overhead")
        # Set ticks
        size_ax.set_yticks([0, 1 / 8, 1 / 4, 1 / 2, 1])

    ax.set_xscale("log", base=2)
    ax.xaxis.set_major_locator(LogLocator(base=4, numticks=20))
    if size:
        ax.set_ylim(-ymax / 6, ymax)
    else:
        ax.set_ylim(0, ymax)
    ax.grid(True, alpha=0.4)
    ax.grid(which="minor", color="gray", alpha=0.2)

    secax = ax.secondary_xaxis("top", functions=(lambda x: x / 4, lambda x: x * 4))
    secax.set_xscale("log", base=2)
    secax.set_xlabel("Array length (u32)")
    secax.set_xticks([4**i for i in range(20)])

    # Drop new/old/best from the legend.
    ax.legend(loc="upper left")
    handles, labels = ax.get_legend_handles_labels()
    new_handles = []
    new_labels = []
    for h, l in zip(handles, labels):
        if l not in ["new", "old", "best", "new_best", "type", "scheme", "Style"]:
            new_handles.append(h)
            new_labels.append(l)
    # Transparent background
    ax.legend(new_handles, new_labels, loc="upper left", framealpha=0.0)

    # Add vertical lines for cache sizes
    for size, name in caches():
        ax.axvline(x=size, color="red", linestyle="--", zorder=0)
        ax.text(size, 0, f"{name} ", color="red", va="bottom", ha="right")

    if l3_size < data.sz.max():
        ax.text(data.sz.max(), 0, "RAM", color="red", va="bottom", ha="right")

    # Save
    if out_format == "svg":
        fig.savefig(f"{store_dir}/{experiment_name}.{out_format}", bbox_inches="tight")
    else:
        fig.savefig(
            f"{store_dir}/{experiment_name}.{out_format}", bbox_inches="tight", dpi=300
        )
    print(f"Saved {experiment_name}.{out_format}")
    # fig.savefig(f"plots/{experiment_name}{human}.svg", bbox_inches="tight")
    # print(f"Saved {experiment_name}{human}.svg")

    # pio.write_html(fig, f"plots/{experiment_name}.html", auto_open=True)
    # Path(f"plots/{experiment_name}.html").write_text(mpld3.fig_to_html(fig))
    # plotly_fig = tls.mpl_to_plotly(fig)
    # plotly.offline.plot(plotly_fig, filename="plotly version of an mpl figure")

    # fig.show()
    return fig


def summary_table(data):
    # Table of statistics at a few key sizes.
    szs = [l1_size / 2, l2_size / 2, l3_size / 3, data.sz.max()]
    names = ["L1", "L2", "L3", "RAM"]
    data = data[data.sz.isin(szs)]
    table = pd.pivot_table(
        data,
        index="name",
        columns="display_size",
        values=["latency", "cycles", "index_size"],
        sort=False,
    )
    print(
        tabulate.tabulate(
            table, headers=table.columns, tablefmt="orgtbl", floatfmt=".1f"
        )
    )


def display_size(size):
    if size <= l1_size:
        return "L1"
    if size <= l2_size:
        return "L2"
    if size <= l3_size:
        return "L3"
    return "RAM"


r1 = re.compile("\\b[a-z_]+::")
r2 = re.compile("(Batched|Full)<(\\d+, )?(Partitioned)?STree<[^<>]*>, (.*)>")


def clean_name(name):
    # strip module names
    name = re.sub(r1, "", name)
    # No type info here
    name = name.replace("::{{closure}}", "")
    # Duplicated 16 from the STree
    name = name.replace("BTreeNode<16>::", "")
    # Drop batch wrapper
    # The batched and full wrappers are not so interesting.
    name = re.sub(r1, "", name)
    name = re.sub(r2, "\\4", name)
    name = name.replace("STree<16, 16>", "STree<>")
    name = name.replace("STree<15, 16>", "STree<B=15>")
    name = name.replace("PartitionedSTree<16, 16, false, false>", "PartitionedSTree<>")
    name = name.replace(
        "PartitionedSTree<16, 16, true, false>", "PartitionedSTree<Compact>"
    )
    name = name.replace("PartitionedSTree<16, 16, false, true>", "PartitionedSTree<L1>")
    name = name.replace("search<128, false>", "search<>")
    name = name.replace("search<128, true>", "search<Prefetch>")
    name = name.replace("search_with_find<find_", "search<find_")
    name = name.replace("splat<128>", "splat")
    name = name.replace("ptr<128>", "ptr")
    name = name.replace("byte_ptr<128>", "byte_ptr")
    name = name.replace("final<128>", "final")
    name = name.replace("final_full<128>", "final_full")
    name = name.replace("skip_prefetch<128, ", "skip_prefetch<SKIP=")
    return name


def read_file(filename):
    data = pd.read_json(filename)

    data["sz"] = data["size"]
    data["scheme"] = data["scheme"].apply(clean_name)
    # Cancat 'name' and 'index' columns.
    data["name"] = (data.scheme + " " + data.params.astype(str)).str.strip()
    data["display_size"] = data["sz"].apply(display_size)

    data["overhead"] = data.index_size / data.sz - 1
    data["capped_overhead"] = [min(o, 1) for o in data.overhead]

    data.latency = data.latency.apply(lambda x: x if x > 0 else 10000)

    def style(scheme):
        if "Partitioned" in scheme:
            return (
                "Partitioned+interleaved" if "interleave" in scheme else "Partitioned"
            )
        else:
            return "Interleaved" if "interleave" in scheme else ""

    data["Style"] = [style(scheme) for scheme in data.scheme]

    global palette

    names = sorted(data.name.unique())
    colors = sns.color_palette(n_colors=20)
    colors += colors
    colors += colors
    colors += colors
    colors += colors
    colors += colors
    colors += colors
    palette = dict(zip(names, colors))

    print(len(names))
    print("Eytzinger:")

    return data


def select(select, all_names, end=False):
    if not isinstance(select, list):
        select = [select]
    selected = []
    for n in all_names:
        for s in select:
            if end:
                if n.endswith(s):
                    selected.append(n)
                    break
            else:
                if s in n:
                    selected.append(n)
                    break
    return selected


# Read all files in the 'results' directory and iterate over them.
def plot_blog():
    all_data = read_file(f"{input_file_prefix}-non-pow2{human}{release}.json")
    data = all_data[all_data.threads == 1]
    all_names = data.name.unique().tolist()
    keep = []

    print(all_names)

    def prune(names):
        nonlocal all_names, keep
        for n in names:
            if n in all_names and n not in keep:
                all_names.remove(n)

    def sel(name, end=False):
        nonlocal all_names
        return select(name, all_names, end)

    spare_names = sel(["LeftMax", "Partitioned"])
    prune(spare_names)

    names = sel(["binary_search", "Eytzinger"])
    plot(
        "1-binary-search",
        "Binary search and Eytzinger layout",
        data,
        names,
        keep,
        latency=True,
        spread=True,
    )
    keep += [names[0], names[-1]]

    names = keep + sel("search<find_linear>")
    plot("2-find-linear", "S-tree", data, names, keep)
    prune(sel("NoHuge"))

    names = keep + sel("search<find")
    plot(
        "3-find", "Optimizing the BTreeNode find function", data, names, keep, ymax=240
    )
    keep.append(names[-1])
    prune(names)

    names = keep + sel("batch<")
    plot("4-batching", "Batch size", data, names, keep, ymax=120)
    keep += sel("batch<128>")
    prune(names)

    names = keep + sel(["batch_prefetch"])
    plot("5-prefetch", "Prefetching", data, names, keep, ymax=60)
    keep.append(names[-1])
    prune(names)

    names = keep + sel(["batch_prefetch<128", "splat", "ptr", "final"])
    plot("6-improvements", "Improvements", data, names, keep, ymax=30)
    keep.append(names[-1])
    prune(names)

    names = keep + sel("skip_prefetch")
    plot("7-skip-prefetch", "Skip prefetch", data, names, keep, new_best=False, ymax=30)
    prune(names)

    names = keep + sel(
        ["interleave_last<64, 2>", "interleave_last<64, 3>", "interleave_all"]
    )
    plot("8-interleave", "Interleave", data, names, keep, ymax=30)
    keep.append(names[-1])
    prune(names)

    # Add construction parameter variants
    all_names += spare_names

    names = keep + select("<>", sel("LeftMax", end=True))

    plot(
        "9-left-max-tree",
        "Left-max-tree",
        data,
        names,
        keep,
        ymax=30,
        size=False,
        highlight=2,
    )
    keep += names[-2:]
    prune(names)

    names = keep + select("<>", sel(["Reverse", "Full"], end=True))
    plot(
        "9-params",
        "Memory layout",
        data,
        names,
        keep,
        new_best=False,
        ymax=30,
        size=False,
        highlight=2,
    )
    prune(names)

    names = keep + select("<B=15>::batch_interleave", sel("LeftMax", end=True))
    plot(
        "10-base15",
        "Base 15",
        data,
        names,
        keep,
        # new_best=False,
        ymax=30,
        size=True,
    )
    last = keep.pop()
    keep.append(names[-1])
    keep.append(last)
    prune(names)

    plot("11-summary", "Summary", data, keep, [], new_best=False, ymax=30)

    names = keep + sel("Simple>::search<>")
    new_best = names[-2]
    plot(
        "20-prefix",
        "Split by prefix",
        data,
        names,
        keep,
        new_best=new_best,
        ymax=30,
        size=True,
    )
    prune(names)
    keep.append(new_best)

    names = keep + sel("Compact>::search<>")
    new_best = names[-2]
    plot(
        "21-compact",
        "Per-tree compact layout",
        data,
        names,
        keep,
        new_best=new_best,
        ymax=30,
        highlight=2,
        size=True,
    )
    prune(names)
    keep.pop()
    keep.append(new_best)

    names = keep + sel("L1>::search<>")
    new_best = names[-2]
    plot(
        "22-l1",
        "L1-compression",
        data,
        names,
        keep,
        new_best=new_best,
        ymax=30,
        highlight=2,
        size=True,
    )
    keep.pop()
    keep.append(new_best)
    prune(names)

    names = keep + sel("Overlapping>::search<>")
    new_best = names[-2]
    plot(
        "23-overlap",
        "Overlapping parts",
        data,
        names,
        keep,
        new_best=new_best,
        ymax=30,
        highlight=2,
        size=True,
    )
    keep.pop()
    keep.append(new_best)
    prune(names)

    names = keep + sel("Map>::search<Prefetch>")
    new_best = names[-2]
    plot(
        "24-map",
        "Prefix-map",
        data,
        names,
        keep,
        new_best=new_best,
        ymax=30,
        highlight=2,
        size=True,
    )
    prune(names)

    keep.pop()
    keep.append(new_best)

    names = keep + sel("Map>::search_interleave")
    new_best = names[-2]
    plot(
        "25-map-interleave",
        "Prefix-map with interleaving",
        data,
        names,
        keep,
        new_best=new_best,
        ymax=30,
        highlight=2,
        size=True,
    )
    keep.append(new_best)
    prune(names)

    # TODO: Human data plot and size comparison.

    plot("27-summary", "Summary", data, keep, [], ymax=30)

    summary_table(data[data.name.isin(keep)])

    data = all_data[all_data.threads == 6]
    plot("28-threads", "6 threads", data, keep, [], ymax=30)


def update_names(names, new_name):
    new_keep = names.copy()
    names.append(new_name)
    print(new_keep)
    return names, new_keep


def plot_binsearch_blog(multithreaded=False):
    # by default, read non-pow2 data
    all_data = read_file(f"{input_file_prefix}-non-pow2{release}.json")
    print(all_data)
    if not multithreaded:
        data = all_data[all_data.threads == 1]
    else:
        data = all_data[all_data.threads == 8]

    multithreaded_suffix = "" if not multithreaded else "-multithreaded"

    all_names = data.name.unique().tolist()
    def size(size):
        return 100 if multithreaded else size

    names = ["SortedVec::binary_search", "SortedVec::binary_search_std"]
    new_best = names[0]
    keep = []
    plot(
        f"binsearch-std-vs-binsearch{multithreaded_suffix}",
        "Naive binary search",
        data,
        names,
        keep,
        new_best=new_best,
        ymax=size(1500),
        highlight=1,
    )

    names, _ = update_names(names, "SortedVec::binary_search_branchless")

    plot(
        f"binsearch-std-vs-branchless{multithreaded_suffix}",
        "Branchless binary search",
        data,
        names,
        keep,
        ymax=size(1500),
        highlight=1,
    )

    names, keep = update_names(names, "SortedVec::binary_search_branchless_prefetch")
    plot(
        f"binsearch-std-vs-branchless-prefetch{multithreaded_suffix}",
        "Branchless binary search with prefetch",
        data,
        names,
        keep,
        ymax=size(1500),
        highlight=1,
    )

    names, keep = update_names(
        names,
        "Batched<16, SortedVec, SortedVec::batch_impl_binary_search_branchless<16>>",
    )
    new_best = names[4]
    plot(
        f"binsearch-std-vs-batched{multithreaded_suffix}",
        "Branchless binary search with batching",
        data,
        names,
        keep,
        new_best=new_best,
        ymax=size(1200),
        highlight=1,
    )

    names = [
        "SortedVec::binary_search_branchless_prefetch",
        "Batched<16, SortedVec, SortedVec::batch_impl_binary_search_branchless<16>>",
        "Batched<16, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<16>>",
    ]
    keep = []
    plot(
        f"binsearch-batched-vs-batched-prefetch{multithreaded_suffix}",
        "Batched binsearch vs batched prefetched binsearch",
        data,
        names,
        keep,
        ymax=size(200),
        highlight=1,
    )

    names = [
        "SortedVec::binary_search_branchless_prefetch",
        "Batched<2, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<2>>",
        "Batched<4, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<4>>",
        "Batched<8, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<8>>",
        "Batched<16, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<16>>",
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<32>>",
        "Batched<64, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<64>>",
        "Batched<128, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<128>>",
    ]
    keep = []
    plot(
        f"binsearch-branchless-prefetched-batched{multithreaded_suffix}",
        "Different batch sizes for branchless batched search",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(1000),
        highlight=1,
    )

    names = [
        "SortedVec::binary_search_branchless_prefetch",
        "Batched<2, SortedVec, SortedVec::batch_impl_binary_search_branchless<2>>",
        "Batched<4, SortedVec, SortedVec::batch_impl_binary_search_branchless<4>>",
        "Batched<8, SortedVec, SortedVec::batch_impl_binary_search_branchless<8>>",
        "Batched<16, SortedVec, SortedVec::batch_impl_binary_search_branchless<16>>",
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless<32>>",
        "Batched<64, SortedVec, SortedVec::batch_impl_binary_search_branchless<64>>",
        "Batched<128, SortedVec, SortedVec::batch_impl_binary_search_branchless<128>>",
    ]
    keep = []
    plot(
        f"binsearch-branchless-batched{multithreaded_suffix}",
        "Different batch sizes for branchless batched search",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(1000),
        highlight=1,
    )

    names = [
        "SortedVec::binary_search_branchless_prefetch",
        "Eytzinger::search",
    ]
    keep = []
    plot(
        f"eytzinger-vs-binsearches{multithreaded_suffix}",
        "Eytzinger search compared to binary search",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(500),
        highlight=1,
    )

    names = [
        "Eytzinger::search",
        "Eytzinger::search_prefetch<2>",
        "Eytzinger::search_prefetch<3>",
        "Eytzinger::search_prefetch<4>",
    ]
    keep = []
    plot(
        f"eytzinger-prefetching{multithreaded_suffix}",
        "Eytzinger layout with prefetching",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(1000),
        highlight=1,
    )

    names = [
        "Eytzinger::search_branchless",
        "Eytzinger::search_branchless_prefetch<4>",
        "Eytzinger::search_prefetch<4>",
    ]
    keep = []
    plot(
        f"eytzinger-branchless-prefetching{multithreaded_suffix}",
        "Eytzinger layout with prefetching",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(500),
        highlight=1,
    )

    names = [
        "Batched<8, Eytzinger, Eytzinger::batch_impl<8>>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl<16>>",
        "Batched<32, Eytzinger, Eytzinger::batch_impl<32>>",
        "Batched<64, Eytzinger, Eytzinger::batch_impl<64>>",
        "Batched<128, Eytzinger, Eytzinger::batch_impl<128>>",
        "Eytzinger::search_branchless_prefetch<4>",
    ]
    keep = []
    plot(
        f"eytzinger-batched-comparison{multithreaded_suffix}",
        "Eytzinger layout with batching",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(500),
        highlight=1,
    )

    names = [
        "Batched<2, Eytzinger, Eytzinger::batch_impl_prefetched<2, 4>>",
        "Batched<4, Eytzinger, Eytzinger::batch_impl_prefetched<4, 4>>",
        "Batched<8, Eytzinger, Eytzinger::batch_impl_prefetched<8, 4>>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<32, Eytzinger, Eytzinger::batch_impl_prefetched<32, 4>>",
        "Batched<64, Eytzinger, Eytzinger::batch_impl_prefetched<64, 4>>",
        "Batched<128, Eytzinger, Eytzinger::batch_impl_prefetched<128, 4>>",
    ]
    keep = []
    plot(
        f"eytzinger-batched-prefetched-comparison{multithreaded_suffix}",
        "Eytzinger layout with batching",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(500),
        highlight=1,
    )

    names = [
        "Eytzinger::search_branchless_prefetch<4>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl<16>>",
    ]
    keep = []
    plot(
        f"eytzinger-best-batching-comparison{multithreaded_suffix}",
        "Comparison of batched Eytzinger - best prefetching & non-prefetching",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(600),
        highlight=1,
    )

    # Binsearch and Eytzinger conclusion
    names = [
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<32>>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
    ]
    print(names)
    keep = []
    plot(
        f"binsearch-eytzinger-conclusion{multithreaded_suffix}",
        "Binary search and Eytzinger layout - best parameters",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(200),
        highlight=1,
    )

    # Interpolation search
    names = ["SortedVec::binary_search_std", "<impl SortedVec>::interpolation_search"]
    keep = []
    plot(
        f"interp-vs-binsearch{multithreaded_suffix}",
        "Interpolation search",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(1500),
        highlight=1,
    )

    # Interpolation search - batching
    names = [
        "<impl SortedVec>::interpolation_search",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<2, SortedVec, <impl SortedVec>::interp_search_batched<2>>",
        "Batched<4, SortedVec, <impl SortedVec>::interp_search_batched<4>>",
        "Batched<8, SortedVec, <impl SortedVec>::interp_search_batched<8>>",
        "Batched<16, SortedVec, <impl SortedVec>::interp_search_batched<16>>",
        "Batched<32, SortedVec, <impl SortedVec>::interp_search_batched<32>>",
    ]
    keep = []
    plot(
        f"interp-vs-binsearch-batched{multithreaded_suffix}",
        "Interpolation search with batching",
        data,
        names,
        keep,
        new_best=False,
        ymax=size(1500),
        highlight=1,
    )

    # power-of-two data for a plot illustrating the pathological case of binary search
    all_data = read_file(f"{input_file_prefix}{release}.json")
    data = all_data[all_data.threads == 1]
    all_names = data.name.unique().tolist()
    names = [
        # "SortedVec::binary_search_branchless_prefetch",
        "Batched<2, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<2>>",
        "Batched<4, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<4>>",
        "Batched<8, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<8>>",
        "Batched<16, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<16>>",
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<32>>",
        "Batched<64, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<64>>",
        "Batched<128, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<128>>",
    ]

    # plot(
    #     "binsearch-branchless-batched-comparison-pow2",
    #     "Different batch sizes for branchless batched search",
    #     data,
    #     names,
    #     keep,
    #     new_best=False,
    #     ymax=2000,
    #     highlight=1,
    # )

    # parallel searching

    all_data = read_file(f"{input_file_prefix}-non-pow2{release}.json")
    data = all_data[all_data.threads == 8]
    # prefetched vs non-prefetched binsearch
    names = [
        "SortedVec::binary_search_branchless_prefetch",
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless<32>>",
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<32>>",
    ]
    keep = []
    plot(
        "binsearch-batched-vs-batched-prefetch-multithreaded",
        "Batched binsearch vs batched prefetched binsearch",
        data,
        names,
        keep,
        ymax=100,
        highlight=1,
    )
    # batch sizes for binsearch
    names = [
        "SortedVec::binary_search_branchless_prefetch",
        "Batched<2, SortedVec, SortedVec::batch_impl_binary_search_branchless<2>>",
        "Batched<4, SortedVec, SortedVec::batch_impl_binary_search_branchless<4>>",
        "Batched<8, SortedVec, SortedVec::batch_impl_binary_search_branchless<8>>",
        "Batched<16, SortedVec, SortedVec::batch_impl_binary_search_branchless<16>>",
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless<32>>",
        "Batched<64, SortedVec, SortedVec::batch_impl_binary_search_branchless<64>>",
        "Batched<128, SortedVec, SortedVec::batch_impl_binary_search_branchless<128>>",
    ]
    keep = []
    plot(
        "binsearch-branchless-prefetched-batched-multithreaded",
        "Different batch sizes for branchless batched search",
        data,
        names,
        keep,
        new_best=False,
        ymax=100,
        highlight=1,
    )
    # prefetched vs non-prefetched eytzinger
    names = [
        "Eytzinger::search_branchless_prefetch<4>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl<16>>",
    ]
    keep = []
    plot(
        f"eytzinger-best-batching-comparison-multithreaded",
        "Comparison of batched Eytzinger - best prefetching & non-prefetching",
        data,
        names,
        keep,
        new_best=False,
        ymax=60,
        highlight=1,
    )
    # eytzinger comparison
    names = [
        "Batched<2, Eytzinger, Eytzinger::batch_impl_prefetched<2, 4>>",
        "Batched<4, Eytzinger, Eytzinger::batch_impl_prefetched<4, 4>>",
        "Batched<8, Eytzinger, Eytzinger::batch_impl_prefetched<8, 4>>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<32, Eytzinger, Eytzinger::batch_impl_prefetched<32, 4>>",
        "Batched<64, Eytzinger, Eytzinger::batch_impl_prefetched<64, 4>>",
        "Batched<128, Eytzinger, Eytzinger::batch_impl_prefetched<128, 4>>",
    ]
    keep = []
    plot(
        f"eytzinger-batched-prefetched-comparison-multithreaded",
        "Eytzinger layout with batching",
        data,
        names,
        keep,
        new_best=False,
        ymax=40,
        highlight=1,
    )

    # eytzinger vs binsearch vs s-trees
    names = [
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<32>>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "STree<>::batch_final LeftMax",
        "STree<>::batch_interleave_all_128 LeftMax"
    ]
    keep = []
    plot(
        f"binsearch-eytzinger-conclusion-multithreaded",
        "Binary search and Eytzinger layout - best parameters",
        data,
        names,
        keep,
        new_best=False,
        ymax=80,
        highlight=1,
    )

    data = all_data
    names = [
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<32>>",
        "Batched<32, SortedVec, SortedVec::batch_impl_binary_search_branchless_prefetch<32>>",
    ]
    keep = []
    plot(
        "single-vs-multithreaded",
        "Single vs. multithreaded search",
        data,
        names,
        keep,
        new_best=False,
        ymax=150,
        highlight=1,
        threads=[1, 8, 1, 8]
    )

    all_data = read_file(f"{input_file_prefix}-non-pow2-human-release.json")
    # 8 physical threads on my machine
    data = all_data[all_data.threads == 1]
    all_names = data.name.unique().tolist()
    # Interpolation search - batching on human data
    names = [
        "<impl SortedVec>::interpolation_search",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<2, SortedVec, <impl SortedVec>::interp_search_batched<2>>",
        "Batched<4, SortedVec, <impl SortedVec>::interp_search_batched<4>>",
        "Batched<8, SortedVec, <impl SortedVec>::interp_search_batched<8>>",
        "Batched<16, SortedVec, <impl SortedVec>::interp_search_batched<16>>",
        "Batched<32, SortedVec, <impl SortedVec>::interp_search_batched<32>>",
    ]
    keep = []
    plot(
        f"interp-vs-binsearch-batched-human-final",
        "Interpolation search with batching",
        data,
        names,
        keep,
        new_best=False,
        ymax=1500,
        highlight=1,
    )

    # 8 physical threads on my machine
    data = all_data[all_data.threads == 8]
    all_names = data.name.unique().tolist()
    # Interpolation search - batching on human data
    names = [
        "<impl SortedVec>::interpolation_search",
        "Batched<16, Eytzinger, Eytzinger::batch_impl_prefetched<16, 4>>",
        "Batched<2, SortedVec, <impl SortedVec>::interp_search_batched<2>>",
        "Batched<4, SortedVec, <impl SortedVec>::interp_search_batched<4>>",
        "Batched<8, SortedVec, <impl SortedVec>::interp_search_batched<8>>",
        "Batched<16, SortedVec, <impl SortedVec>::interp_search_batched<16>>",
        "Batched<32, SortedVec, <impl SortedVec>::interp_search_batched<32>>",
    ]
    keep = []
    plot(
        "interp-vs-binsearch-batched-human-final-multithreaded",
        "Interpolation search with batching",
        data,
        names,
        keep,
        new_best=False,
        ymax=200,
        highlight=1,
    )


def plot_interp_search_test():
    all_data = read_file(f"{input_file_prefix}-non-pow2{human}{release}.json")
    data = all_data[all_data.threads == 1]
    all_names = data.name.unique().tolist()
    print(all_names)

    names = all_names
    keep = []

    plot(
        "interp-search-vs-binsearch",
        "Naive interpolation search",
        data,
        names,
        keep,
        new_best=False,
        ymax=1500,
        highlight=1,
    )


def filter_large(data, x=2.5):
    data["latency"] = data.apply(
        lambda row: (row.latency if row.overhead < x else 2**40), axis=1
    )
    data = data[data.sz > 2**12]
    return data


def plot_all():
    data = read_file(f"results/results.json")
    data = data[data.threads == 6]
    # data = filter_large(data, 1.25)
    all_names = data.name.unique().tolist()
    print(all_names)
    print("Plotting...")
    names = select("<false>", all_names)
    plot("99-all", "ALL", data, names, [], ymax=30, size=False)
    names = select("<true>", all_names)
    plot("99-all", "ALL", data, names, [], ymax=30, size=False)
    return


# plt.style.use("dark_background")
# plt.close("all")
parser = argparse.ArgumentParser()
parser.add_argument("--release", action="store_true")
parser.add_argument("--human", action="store_true")
parser.add_argument(
    "--store_dir", default="", type=str, help="Directory to store the plots"
)
parser.add_argument(
    "--input_file_prefix",
    default="results/results",
    type=str,
    help="Input file to read the data from",
)
parser.add_argument(
    "--out_format", default="png", type=str, help="Output format of the plots"
)
parser.add_argument("--interp_search_test", action="store_true")

args = parser.parse_args()
if args.release:
    release = "-release"

if args.human:
    human = "-human"
store_dir = args.store_dir
input_file_prefix = args.input_file_prefix
out_format = args.out_format

if args.interp_search_test:
    plot_interp_search_test()
else:
    plot_binsearch_blog()
    # plot_binsearch_blog(multithreaded=True)
# plot_all()
