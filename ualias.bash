# bash completion for ualias

_ualias()
{
	local cur prev words cword
	_init_completion || return

	if [[ $cur == -* ]]; then
		COMPREPLY=($(compgen -W '--vendor --product --serial --name --ucfirst --help --version -w -p -s -n -u -h' -- "$cur"))
		return
	fi

	case $prev in
		-w|--vendor|-p|--product|-s|--serial|-n|--name)
			return
			;;
	esac
}

complete -F _ualias ualias
