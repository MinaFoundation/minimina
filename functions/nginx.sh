#!/usr/bin/env bash
set -e

generate-nginx-config() {


echo """

worker_processes  1;
pid               /root/nginx.pid;

events {
  worker_connections  1024;
}

http {
  include             mime.types;
  default_type        application/octet-stream;
  sendfile            on;
  keepalive_timeout   65;

  server {
    listen            8080;
    server_name       localhost;

    location / {
      root   html;
      index  index.html index.htm;
    }
    error_page  500 502 503 504 /50x.html;
    location = /50x.html {
      root  html;
    }

    location /whale_0/graphql {
      if ($request_method = OPTIONS ) {
        add_header Allow "POST, OPTIONS";
        add_header Access-Control-Allow-Headers "Origin, X-Requested-With, Content-Type, Accept";
        add_header Access-Control-Allow-Origin "*";

        return 200;
      }

      ###PROXY_PASS###
      proxy_pass http://localhost:4005/graphql;
      #proxy_set_header	Access-Control-Allow-Origin *;
      proxy_set_header Origin $http_origin;
      proxy_hide_header Access-Control-Allow-Origin;
      add_header Access-Control-Allow-Origin *;
    }

    location /whale_1/graphql {
      if ($request_method = OPTIONS ) {
        add_header Allow "POST, OPTIONS";
        add_header Access-Control-Allow-Headers "Origin, X-Requested-With, Content-Type, Accept";
        add_header Access-Control-Allow-Origin "*";

        return 200;
      }

      ###PROXY_PASS###
      proxy_pass http://localhost:4005/graphql;
      #proxy_set_header	Access-Control-Allow-Origin *;
      proxy_set_header Origin $http_origin;
      proxy_hide_header Access-Control-Allow-Origin;
      add_header Access-Control-Allow-Origin *;
    }


  }

  include servers/*;
}

"""

}
