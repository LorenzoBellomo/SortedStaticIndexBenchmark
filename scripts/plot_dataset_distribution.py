
import numpy as np
import matplotlib as plt
import pandas as pd

def plot_dataset(dataset, **kwargs):
    f = open(dataset, "r")
    data = np.fromfile(f, dtype=np.uint64)
    df = pd.DataFrame({'data': data}).reset_index(drop=True)
    df.plot(legend=False, **kwargs)
    
    f.close()

datasets = ["books_200M_uint32", "companynet_uint32", "fb_200M_uint64", "lognormal_uint32", "normal_uint32", 
            "wiki_ts_200M_uint32", "zipf_uint32", "books_800M_uint64", "exponential_uint32", "friendster_50M_uint32", 
            "osm_cellids_800M_uint64", "wiki_ts_200M_uint64"]

for dataset in datasets:
    plot_dataset(dataset)