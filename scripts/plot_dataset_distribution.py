
import numpy as np
import matplotlib as plt
import pandas as pd
import os

def plot_dataset(path_to_write, path_to_read, dataset, **kwargs):
    f = open("{}/{}".format(path_to_read, dataset), "r")
    if dataset.endswith("2"):
        print("uint32: ", dataset)
        data = np.fromfile(f, dtype=np.uint32)
    else:
        print("uint64: ", dataset)
        data = np.fromfile(f, dtype=np.uint64)
    df = pd.DataFrame({'data': data}).reset_index(drop=True)
    df.plot(legend=False, **kwargs).get_figure().savefig("{}/{}.png".format(path_to_write, dataset))
    f.close()

datasets = ["zipf_uint32", "books_200M_uint32", "companynet_uint32", "fb_200M_uint64", "lognormal_uint32", "normal_uint32", 
            "wiki_ts_200M_uint32", "books_800M_uint64", "exponential_uint32", "friendster_50M_uint32", 
            "osm_cellids_800M_uint64", "wiki_ts_200M_uint64"]

path_to = "output/plots/dataset_distributions"
if not os.path.exists(path_to):
    os.makedirs(path_to)

for dataset in datasets:
    plot_dataset(path_to, "data", dataset)