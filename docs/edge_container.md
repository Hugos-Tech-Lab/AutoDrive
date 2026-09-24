# Edge container
- runs on PC in lab (in rust)
- knows addresses and ids of all cars and requests diagnostic metrics
- might not need ID in request as it can be derived from the IP address
- forwards scripts from cloud to chips (logic for queueing is on cloud)
- forwards data from chips to grafana
- grafana is hosted externelly
- devices can publish sensor, motor, chip data to grafana (and to the cloud) via edge
