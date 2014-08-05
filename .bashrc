# .bashrc

# Source global definitions
if [ -f /etc/bashrc ]; then
	. /etc/bashrc
fi

##
## Sweet Git helpers
##

# Add my Git branch to my prompt
if [ -f ~/dotfiles/.git-prompt.sh ]; then
    source ~/dotfiles/.git-prompt.sh
fi

if hash __git_ps1 2>/dev/null; then
    PS1="\[$GREEN\]\t\[$RED\]-\[$BLUE\]\u\[$YELLOW\]\[$YELLOW\]\w\[\033[m\]\[$MAGENTA\]\$(__git_ps1)\[$WHITE\]\$ "
else
    export PS1='\[\033[01;32m\]\u@\h\[\033[00m\] \[\033[01;34m\]\w\[\033[00m\]$(git branch &>/dev/null; if [ $? -eq 0 ]; then echo "\[\033[01;33m\]($(git branch | grep ^*|sed s/\*\ //))\[\033[00m\]"; fi)$ '
fi

# Git auto-complete
if [ -f ~/dotfiles/tools/.git-completion.bash ]; then
    . ~/dotfiles/tools/.git-completion.bash
fi

### Added by the Heroku Toolbelt
export PATH="/usr/local/heroku/bin:$PATH"
