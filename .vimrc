""
"" Customisations
""

source ~/.vimrc.before
execute pathogen#infect()
execute pathogen#helptags()
source ~/.vimrc.after

" Custom file per system
if filereadable(glob("~/.vimrc.custom")) 
    source ~/.vimrc.custom
endif
