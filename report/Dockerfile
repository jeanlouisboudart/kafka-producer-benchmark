 FROM jupyter/datascience-notebook
 # necessary as we will be producing files within the container
 ARG UID=1000
 ARG GID=1000
 USER root
 RUN groupadd -g $GID jovyan
 RUN usermod -u $UID -g $GID jovyan 
 USER jovyan
 RUN pip install plotly kaleido
 CMD jupyter nbconvert work/benchmark-report.ipynb --execute --to markdown --output report.md --no-input