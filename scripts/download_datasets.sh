# MOST OF THE CODE HERE COMES FROM SOSD -> https://github.com/learnedsystems/SOSD

# Calculate md5 checksum of FILE and stores it in MD5_RESULT
function get_checksum() {
   FILE=$1
   if [ -x "$(command -v md5sum)" ]; then
      # Linux
      MD5_RESULT=`md5sum ${FILE} | awk '{ print $1 }'`
   else
      # OS X
      MD5_RESULT=`md5 -q ${FILE}`
   fi
}

function download_file_zst() {
   FILE=$1;
   CHECKSUM=$2;
   URL=$3;

   # Check if file already exists
   if [ -f ${FILE} ]; then
      # Exists -> check the checksum
      get_checksum ${FILE}
      if [ "${MD5_RESULT}" != "${CHECKSUM}" ]; then
         wget -O - ${URL} | zstd -d > ${FILE}
      fi
   else
      # Does not exists -> download
      wget -O - ${URL} | zstd -d > ${FILE}
   fi
   # Validate (at this point the file should really exist)
   get_checksum ${FILE}
   if [ "${MD5_RESULT}" != "${CHECKSUM}" ]; then
      echo "error checksum does not match: run download again"
      exit -1
   else
      echo ${FILE} "checksum ok"
   fi
}

function main() {
   echo "downloading data ..."
   mkdir -p data
   mkdir -p data/lookups
   cd data

   # Format: download_file <file_name> <md5_checksum> <url>
   if [ ! -f wiki_ts_200M_uint64 ]; then
      download_file_zst wiki_ts_200M_uint64 4f1402b1c476d67f77d2da4955432f7d https://dataverse.harvard.edu/api/access/datafile/:persistentId?persistentId=doi:10.7910/DVN/JGVF9A/SVN8PI
   fi
   
   if [ ! -f books_200M_uint32 ]; then
      download_file_zst books_200M_uint32 9f3e578671e5c0348cdddc9c68946770 https://dataverse.harvard.edu/api/access/datafile/:persistentId?persistentId=doi:10.7910/DVN/JGVF9A/5YTV8K  
   fi

   if [ ! -f books_800M_uint64 ]; then
      download_file_zst books_800M_uint64 8708eb3e1757640ba18dcd3a0dbb53bc https://www.dropbox.com/s/y2u3nbanbnbmg7n/books_800M_uint64.zst?dl=1
   fi

   if [ ! -f osm_cellids_800M_uint64 ]; then
      download_file_zst osm_cellids_800M_uint64 70670bf41196b9591e07d0128a281b9a https://www.dropbox.com/s/j1d4ufn4fyb4po2/osm_cellids_800M_uint64.zst?dl=1
   fi

   if [ ! -f fb_200M_uint64 ]; then
      download_file_zst fb_200M_uint64 3b0f820caa0d62150e87ce94ec989978 https://dataverse.harvard.edu/api/access/datafile/:persistentId?persistentId=doi:10.7910/DVN/JGVF9A/EATHF7 
   fi

   # downloading now NON SOSD datasets
   if [ ! -f companynet_uint32 ]; then
      wget -O companynet_uint32 https://www.dropbox.com/scl/fi/pmrx9xwl6qchd0rm11x90/companynet_uint32?rlkey=mvnyl5cicjuu6htjywslwx7wa&st=c17j7k89&dl=0
   fi
   if [ ! -f friendster_50M_uint32 ]; then
      wget -O friendster_50M_uint32 https://www.dropbox.com/scl/fi/tuesjan0qikkewij9i8vi/friendster_50M_uint32?rlkey=uqj459q30v2ipcidzjp3293pa&st=2jmcygqf&dl=0
   fi

   # GENERATING ALL THE OTHER DATASETS
   cd ../scripts
   g++ -std=c++11 -I . -O3 elaborate_datasets.cpp -o elaborate_datasets
   echo "GENERATING SYNTETHIC DATASETS, AND PRINTING ALL THE STATS"
   ./elaborate_datasets > ../output/dataset_stats.txt

   rm elaborate_datasets
   cd ..
   echo "done"
}

# Run
main