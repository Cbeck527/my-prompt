# Check if my-prompt is installed and available for beta testing
if command -v my-prompt &>/dev/null
    set -g fish_transient_prompt 1
    function fish_prompt
        my-prompt --code $status $argv
    end
end
