* Create an nginx file to route ssl to any non ssl webapp  
for example, `/etc/nginx/sites-enabled/mySubdomain.holochain.net`

here is an example template:
```nginx
server {
   listen         80;

##### set the server_name to the subdomain you have chosen for your webapp
   server_name    chat.holochain.net;
################

    if ($scheme != "https") {
        return 301 https://$host$request_uri;
    } # managed by Certbot

}

### set the upstream name to the name of the webapp, e.g. mattermost on port 81
upstream mattermost81 {
    #### leave the server at 127.0.0.1
    #### set the port to the port the webapp is running on
    server 127.0.0.1:81;
    #server 127.0.0.1:5001;
  }

server {
########## set the log dir to your chosen log directory
### maybe turn off debug when you are ready? (big files)
  error_log /home/###my web apps user directory####/logs/error.log debug;
##########

  listen 443;
  ### set the server name to the subdomain you wish to address the webapp, e.g. "chat.holochain.net"
  server_name chat.holochain.net;
  
############################
### set the root directory
root /home/webappdir/publicDirectory;
############################

ssl on;
############################
### Certificate
### make these with certbot, instructions: https://certbot.eff.org/#ubuntuxenial-nginx
############################
ssl_certificate /etc/letsencrypt/live/### my URL DOmain, e.g. chat.holochain.net ####/fullchain.pem; # managed by Certbot
ssl_certificate_key /etc/letsencrypt/live/### my URL DOmain, e.g. chat.holochain.net ####/privkey.pem; # managed by Certbot
############################


  ssl_session_timeout 5m;

  ssl_protocols SSLv3 TLSv1;
  ssl_ciphers ALL:!ADH:!EXPORT56:RC4+RSA:+HIGH:+MEDIUM:+LOW:+SSLv3:+EXP;
  ssl_prefer_server_ciphers on;

   location / {
       client_max_body_size 50M;
       proxy_set_header Connection "";
       proxy_set_header Host $http_host;
       proxy_set_header X-Real-IP $remote_addr;
       proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
       proxy_set_header X-Forwarded-Proto $scheme;
       proxy_set_header X-Frame-Options SAMEORIGIN;
       proxy_buffers 256 16k;
       proxy_buffer_size 16k;
       proxy_read_timeout 600s;
       # proxy_cache mattermost_cache;
       # proxy_cache_revalidate on;
       # proxy_cache_min_uses 2;
       # proxy_cache_use_stale timeout;
       # proxy_cache_lock on;

##### this must match the name you chose for the upstream
       proxy_pass http://mattermost81;
#####################
   }	
}
```

* run certbot with sudo, as in the instructions here: https://certbot.eff.org/#ubuntuxenial-nginx

* run `sudo service nginx reload`

