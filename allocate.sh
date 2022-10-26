
curl --key client.key \
     --cert client.crt \
     --cacert ca.crt \
     -H "Content-Type: application/json" \
     --data '{"namespace":"'default'"}' \
     https://10.103.94.74/gameserverallocation \
     -X POST

curl -H "Content-Type: application/json" \
     --data '{"namespace":"'default'"}' \
     http://agones-allocator.agones-system.svc.cluster.local/gameserverallocation \
     -X POST

curl -H "Content-Type: application/json" \
     --data '{
               "namespace":"'default'",
               "gameServerSelectors" : [
                    {
                         "matchLabels": { "agones.dev/sdk-gs-session-ready": "true" },
                         "gameServerState": 1
                    }
               ]
          }' \
     http://agones-allocator.agones-system.svc.cluster.local/gameserverallocation \
     -X POST