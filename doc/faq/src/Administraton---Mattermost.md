## Mattermost Server
* server address
  * 162.243.136.142
* username
  * mattermost
* password
  * https://docs.google.com/spreadsheets/d/1S-bHJM3kTJKij_C-mXlyYR-Nyb2nEeGxUOJwnE2lnA8/edit#gid=0
* notes
  * mattermost and the integrations are all running within screen
  * to connect to the screen session
    * $HOME/Scripts/screen.mattermost

### Integrations
* Zapier
  * github integration
  
* mattermost-github - DID NOT USE - OBSOLETE IGNORE
  * https://github.com/softdevteam/mattermost-github-integration
  * cloned to
    * $HOME/mattermost-github
  * other installation
    * docker
  * config.py
```python
USERNAME = "Github"
ICON_URL = "yourdomain.org/github.png"
MATTERMOST_WEBHOOK_URLS = {
    'default' : ("http://chat.holochain.net/hooks/jtk9idg4ifnj7ncm9et6tjdi5w", "off-topic"),
    'metacurrency/holochain' : ("http://chat.holochain.net/hooks/jtk9idg4ifnj7ncm9et6tjdi5w", "feedcode"),
    #'teamname' : ("yourdomain.org/hooks/hookid3", "town-square"),
    #'teamname/unimportantrepo' : None,
}
GITHUB_IGNORE_ACTIONS = {
    "issues": ["labeled", "assigned"],
}
SECRET = None
SHOW_AVATARS = True
SERVER = {
    'hook': "/"
,   'address': "0.0.0.0"
,   'port': 5000
}
```
  * github webhook
    * payload url
      * http://chat.holochain.net:5000
    * secret
      * $$BLANK$$
      * setting secret to anything other than blank causes the flask startup to fail
    * which events
      * send me everything
