# This function removes the discord token from your environment and 
# deletes itself from your global scope, as well. To envoke it, type
# ./env.ps1 to start it. Once active, you can type deactivate to 
# start the function.
function global:deactivate ([switch] $NonDestructive) {
    # Remove discord token from environment variables
    if ($env:DISCORD_TOKEN) {
        Remove-Item env:DISCORD_TOKEN -ErrorAction SilentlyContinue
    }
    # Remove this function from the global scope
    if (!$NonDestructive) {
        Remove-Item function:deactivate
    }
}

# Enter your discord token here for development
$env:DISCORD_TOKEN = ""