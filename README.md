# Sensordrivers
Currently supporting:

AM2302
BH1750
BMP280

# What it does
This small rust application reads out the sensors connected via I2C / GPIO in a given interval. The values gets send via an UDP socket to the [APIServer][APIServer]. Right now it is a mixture between bare metal programming and the usement of external crates. The final goal is to get rid of all those crated to achieve independence. This becomes interesting when migrating this application from the Raspberry Pi 4 to a STM32 microcontroller which will be running the [NextLevelRTOS] .
# Progress

## Implemented
* Read out the sensor data and send it over UDP to an APIServer running on localhost

## Todo
* Get rid of external (std)-crates

# Annotations
This application does not run stand alone and depends from this repositories:

[WeatherGui][WeatherGui]

[APIServer][APIServer]

[WeatherGui]: (https://github.com/wolfbiker1/weatherGui)
[APIServer]: (https://github.com/wolfbiker1/weatherStationAPIServer)
[NextLevelRTOS]: (https://github.com/wolfbiker1/NextLevelRTOS)