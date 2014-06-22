# .bashrc

# Source global definitions
if [ -f /etc/bashrc ]; then
	. /etc/bashrc
fi

# User specific aliases and functions
alias ll='ls -alh'
alias t='tmux'
alias ta='tmux a'

##
## Sweet Git helpers
##

# Add my Git branch to my prompt
if [ -f ~/dotfiles/.git-prompt.sh ]; then
    source ~/dotfiles/.git-prompt.sh
fi
PS1="\[$GREEN\]\t\[$RED\]-\[$BLUE\]\u\[$YELLOW\]\[$YELLOW\]\w\[\033[m\]\[$MAGENTA\]\$(__git_ps1)\[$WHITE\]\$ "


# Git auto-complete
if [ -f ~/dotfiles/tools/.git-completion.bash ]; then
    . ~/dotfiles/tools/.git-completion.bash
fi
