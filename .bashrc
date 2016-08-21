# .bashrc

# Source global definitions
if [ -f /etc/bashrc ]; then
	. /etc/bashrc
fi

# Set default editors to vim (not vi)
export VISUAL=vim;
export EDITOR=vim;

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

# docker-osx-dev
export DOCKER_HOST=tcp://192.168.59.103:2376
export DOCKER_CERT_PATH=/Users/jmcfarland/.boot2docker/certs/boot2docker-vm
export DOCKER_TLS_VERIFY=1

# docker-osx-dev
export DOCKER_HOST=tcp://192.168.59.103:2376
export DOCKER_CERT_PATH=/Users/jmcfarland/.boot2docker/certs/boot2docker-vm
export DOCKER_TLS_VERIFY=1

# docker-osx-dev
export DOCKER_HOST=tcp://192.168.59.103:2376
export DOCKER_CERT_PATH=/Users/jmcfarland/.boot2docker/certs/boot2docker-vm
export DOCKER_TLS_VERIFY=1

# docker-osx-dev
export DOCKER_CERT_PATH=/Users/jmcfarland/.boot2docker/certs/boot2docker-vm
export DOCKER_TLS_VERIFY=1
export DOCKER_HOST=tcp://192.168.59.103:2376
