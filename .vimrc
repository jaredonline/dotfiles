""
"" Customisations
""

source ~/.vimrc.before
execute pathogen#infect()
execute pathogen#helptags()
source ~/.vimrc.after

" Custom to each platform
if filereadable("~/.vimrc.custom")
    source ~/.vimrc.custom
endif
