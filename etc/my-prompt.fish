# Check if my-prompt is installed and available for beta testing
if command -v my-prompt &>/dev/null
    set -g fish_transient_prompt 1

    function __my_prompt_refresh_direnv_status
        if not set -q DIRENV_FILE; or not set -q DIRENV_WATCHES
            set -e __my_prompt_direnv_file
            set -e __my_prompt_direnv_watches
            set -e __my_prompt_direnv_status_json
            return
        end

        if set -q __my_prompt_direnv_file __my_prompt_direnv_watches
            if test "$DIRENV_FILE" = "$__my_prompt_direnv_file"; and test "$DIRENV_WATCHES" = "$__my_prompt_direnv_watches"
                return
            end
        end

        set -g __my_prompt_direnv_file "$DIRENV_FILE"
        set -g __my_prompt_direnv_watches "$DIRENV_WATCHES"

        set -l status_json (direnv status --json 2>/dev/null | string collect)
        set -l direnv_status $pipestatus[1]

        if test $direnv_status -eq 0
            set -g __my_prompt_direnv_status_json "$status_json"
        else
            set -e __my_prompt_direnv_status_json
        end
    end

    function fish_prompt
        set -l previous_status $status
        __my_prompt_refresh_direnv_status

        set -lx MY_PROMPT_DIRENV_STATUS_JSON ""
        if set -q __my_prompt_direnv_status_json
            set MY_PROMPT_DIRENV_STATUS_JSON "$__my_prompt_direnv_status_json"
        end

        my-prompt --code $previous_status $argv
    end
end
