# ads-client

Communicate with a Beckhoff PLC or other devices that communicate with [ADS](https://download.beckhoff.com/download/document/automation/twincat3/TwinCAT_3_ADS_INTRO_EN.pdf).
This client implementation builds on [ads-proto](https://github.com/wyda/ads-proto)

All ADS commands are supported. Additionally sumup commands for read and write are implemented. 
The sumup requests will bundle multiple read or write request into a single one reducing the traffic.
It is also possible to create/customize requests manually and supply them to the request methode (request_example.rs).

All requests will return the complete response data. You may want to checkout ads-proto to get more information on this.

To get started i recommend checking out the examples. 
If you want to run the examples you will need a running TwinCat PLC or another ADS device and you probably want to customize the connection details and var names.
If you want to connect to a TwinCat PLC that runs on a remote devive make sure you add a route on that device allowing you to connect.