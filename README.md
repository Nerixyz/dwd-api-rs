# DWD API

This is a wrapper around the data provided by the [DWD](https://www.dwd.de/) on [opendata.dwd.de](https://opendata.dwd.de/).

## Why is there a need for this API?

The data provided has an odd shape for modern interfaces and isn't really documented.

# API Documentation

## `GET /forecast/{station}`

* `{station}` - The station id - either obtained by `GET /stations` or searching [here](https://www.dwd.de/DE/leistungen/met_verfahren_mosmix/mosmix_stationskatalog.cfg?view=nasPublication). The station id is static and won't change.

### Response

```typescript
interface ForecastResponse {
    name: string;
    description: string;
    issuer: string;
    generating_process: string;
    issue_time: timestamp_ms;
    reference_models: Array<{
        name: string,
        reference_time: timestamp_ms,
    }>;
    coordinates: string;
    data: ForecastResponseData;
    n_data_points: number;
}
```

<details>
<summary>ForecastResponseData</summary>

The response data is an object where all the values are an array of the **same** length.
It can be viewed as a table-like structure:
```typescript
// type ForecastElements = 'temp' | 'dew_point' .... (see table below)

type ForecastResponseData = 
    { [Key in ForecastElements]?: Array<float32 | null>; } & { time_steps: timestamp_ms[]; }
// some elements in the array may be null because they either don't have a value _or_ the value is updated less than other values.


// Note: in e.g. JavaScript (ans JSON) all the values are represented as a `number`, but for other languages that might not be true.
``` 
The property `time_steps` is always present. 

### Properties

For more information take a look at [this](https://opendata.dwd.de/weather/lib/MetElementDefinition.xml).

**Naming**
* `p_*` > probability
* `e_*` > absolute error of ...
* `_{n}h` > last `{n}` hours
* `p{n}{unit}` > 0.`{n}{unit}`

**Units**
* `p` > `%`
* `mm` > millimeters
* `ft` > feet
* `kn` > knots

| Property | Unit | Description | DWD-Name |
| ---------|------|-------------|----------|
|`temp`|K|Temperature 2m above surface|TTT|
|`dew_point`|K|Dewpoint 2m above surface|Td|
|`max_temp`|K|Maximum temperature - within the last 12 hours|TX|
|`min_temp`|K|Minimum temperature - within the last 12 hours|TN|
|`wind_direction`|0°..360°|Wind direction|DD|
|`wind_speed`|m/s|Wind speed|FF|
|`max_wind_gust_1h`|m/s|Maximum wind gust within the last hour|FX1|
|`max_wind_gust_3h`|m/s|Maximum wind gust within the last 3 hours|FX3|
|`max_wind_gust_12h`|m/s|Maximum wind gust within the last 12 hours|FXh|
|`precipitation_1h_significant_weather`|kg/m2|Total precipitation during the last hour consistent with significant weather|RR1c|
|`precipitation_1h`|kg/m2|Total precipitation during the last hour|RR1|
|`precipitation_3h_significant_weather`|kg/m2|Total precipitation during the last 3 hours consistent with significant weather|RR3c|
|`precipitation_3h`|kg/m2|Total precipitation during the last 3 hours|RR3|
|`snow_rain_eq_1h`|kg/m2|Snow-Rain-Equivalent during the last hour|RRS1c|
|`snow_rain_eq_3h`|kg/m2|Snow-Rain-Equivalent during the last 3 hours|RRS3c|
|`significant_weather`|-|Significant Weather|ww|
|`past_weather_6h`|-|Past weather during the last 6 hours|W1W2|
|`total_cloud_cover`|% (0..100)|Total cloud cover|N|
|`effective_cloud_cover`|% (0..100)|Effective cloud cover|Neff|
|`cloud_cover_500ft`|% (0..100)|Cloud cover below 500 ft.|N05|
|`low_cloud_cover`|% (0..100)|Low cloud cover (lower than 2 km)|Nl|
|`midlevel_cloud_cover`|% (0..100)|Midlevel cloud cover (2-7 km)|Nm|
|`high_cloud_cover`|% (0..100)|High cloud cover (>7 km)|Nh|
|`surface_pressure`|Pa|Surface pressure, reduced|PPPP|
|`temp_5cm`|K|Temperature 5cm above surface|T5cm|
|`shortwave_radiation_3h`|kJ/m2|Short wave radiation balance during the last 3 hours|RadS3|
|`global_irradiance`|kJ/m2|Global Irradiance|Rad1h|
|`longwave_radiation_3h`|kJ/m2|Long wave radiation balance during the last 3 hours|RadL3|
|`visibility`|m|Visibility|VV|
|`sunshine_last_hour`|s|Sunshine duration during the last Hour|SunD1|
|`p_wind_gust_25kn_12h`|% (0..100)|Probability of wind gusts >= 25kn within the last 12 hours|FXh25|
|`p_wind_gust_40kn_12h`|% (0..100)|Probability of wind gusts >= 40kn within the last 12 hours|FXh40|
|`p_wind_gust_55kn_12h`|% (0..100)|Probability of wind gusts >= 55kn within the last 12 hours|FXh55|
|`p_fog_1h`|% (0..100)|Probability for fog within the last hour|wwM|
|`p_fog_6h`|% (0..100)|Probability for fog within the last 6 hours|wwM6|
|`p_fog_12h`|% (0..100)|Probability for fog within the last 12 hours|wwMh|
|`p_precipitation_0mm_12h`|% (0..100)|Probability of precipitation > 0.0mm during the last 12 hours|Rh00|
|`p_precipitation_p2mm_6h`|% (0..100)|Probability of precipitation > 0.2mm during the last 6 hours|R602|
|`p_precipitation_p2mm_12h`|% (0..100)|Probability of precipitation > 0.2mm during the last 12 hours|Rh02|
|`p_precipitation_p2mm_24h`|% (0..100)|Probability of precipitation > 0.2mm during the last 24 hours|Rd02|
|`p_precipitation_1mm_12h`|% (0..100)|Probability of precipitation > 1.0mm during the last 12 hours|Rh10|
|`p_precipitation_5mm_6h`|% (0..100)|Probability of precipitation > 5.0mm during the last 6 hours|R650|
|`p_precipitation_5mm_12h`|% (0..100)|Probability of precipitation > 5.0mm during the last 12 hours|Rh50|
|`p_precipitation_5mm_24h`|% (0..100)|Probability of precipitation > 5.0mm during the last 24 hours|Rd50|
|`min_temp_5cm_12h`|K|Minimum surface temperature at 5cm within the last 12 hours|TG|
|`mean_temp_24h`|K|Mean temperature during the last 24 hours|TM|
|`precipitation_duration_1h`|s|Duration of precipitation within the last hour|DRR1|
|`p_drizzle_1h`|% (0..100)|Probability: Occurrence of drizzle within the last hour|wwZ|
|`p_straitform_precipitation_1h`|% (0..100)|Probability: Occurrence of stratiform precipitation within the last hour|wwD|
|`p_convective_precipitation_1h`|% (0..100)|Probability: Occurrence of convective precipitation within the last hour|wwC|
|`p_thunderstorms_1h`|% (0..100)|Probability: Occurrence of thunderstorms within the last hour|wwT|
|`p_liquid_precipitation_1h`|% (0..100)|Probability: Occurrence of liquid precipitation within the last hour|wwL|
|`p_solid_precipitation_1h`|% (0..100)|Probability: Occurrence of solid precipitation within the last hour|wwS|
|`p_freezing_rain_1h`|% (0..100)|Probability: Occurrence of freezing rain within the last hour|wwF|
|`p_precipitation_1h`|% (0..100)|Probability: Occurrence of precipitation within the last hour|wwP|
|`p_visibility_below_1km`|% (0..100)|Probability: Visibility below 1000m|VV10|
|`e_temp`|K|Absolute error temperature 2m above surface|E_TTT|
|`e_wind_speed`|m/s|Absolute error wind speed 10m above surface|E_FF|
|`e_wind_direction`|0Â°..360Â°|Absolute error wind direction|E_DD|
|`e_dew_point`|K|Absolute error dew point 2m above surface|E_Td|
|`precipitation_6h`|kg / m2|Total precipitation during the last 6 hours|RR6|
|`precipitation_6h_significant_weather`|kg / m2|Total precipitation during the last 6 hours consistent with significant weather|RR6c|
|`p_precipitation_0mm_6h`|% (0..100)|Probability of precipitation > 0.0mm during the last 6 hours|R600|
|`p_precipitation_p1mm_1h`|% (0..100)|Probability of precipitation > 0.1 mm during the last hour|R101|
|`p_precipitation_p2mm_1h`|% (0..100)|Probability of precipitation > 0.2 mm during the last hour|R102|
|`p_precipitation_p3mm_1h`|% (0..100)|Probability of precipitation > 0.3 mm during the last hour|R103|
|`p_precipitation_p5mm_1h`|% (0..100)|Probability of precipitation > 0.5 mm during the last hour|R105|
|`p_precipitation_p7mm_1h`|% (0..100)|Probability of precipitation > 0.7 mm during the last hour|R107|
|`p_precipitation_1mm_1h`|% (0..100)|Probability of precipitation > 1.0 mm during the last hour|R110|
|`p_precipitation_2mm_1h`|% (0..100)|Probability of precipitation > 2.0 mm during the last hour|R120|
|`sunshine_duration_yesterday`|s|Yesterdays total sunshine duration |SunD|
|`rel_sunshine_duration_24h`|% (0..100)|Relative sunshine duration within the last 24 hours|RSunD|
|`p_rel_sunshine_duration_24h`|% (0..100)|Probability: relative sunshine duration >  0 % within 24 hours|PSd00|
|`p_rel_sunshine_duration_30p_24h`|% (0..100)|Probability: relative sunshine duration > 30 % within 24 hours|PSd30|
|`p_rel_sunshine_duration_60p_24h`|% (0..100)|Probability: relative sunshine duration > 60 % within 24 hours|PSd60|
|`global_irradiance_1h`|% (0..80)|Global irradiance within the last hour|RRad1|
|`potential_evapotranspiration_24h`|kg / m2|Potential evapotranspiration within the last 24 hours|PEvap|
|`p_precipitation_3mm_1h`|% (0..100)|Probability of precipitation > 3.0 mm during the last hour|R130|
|`p_precipitation_5mm_1h`|% (0..100)|Probability of precipitation > 5.0 mm during the last hour|R150|
|`p_precipitation_10mm_1h`|% (0..100)|Probability of precipitation > 10 mm during the last hour|RR1o1|
|`p_precipitation_15mm_1h`|% (0..100)|Probability of precipitation > 15 mm during the last hour|RR1w1|
|`p_precipitation_25mm_1h`|% (0..100)|Probability of precipitation > 25 mm during the last hour|RR1u1|
|`p_straightform_precipitation_6h`|% (0..100)|Probability: Occurrence of stratiform precipitation within the last 6 hours|wwD6|
|`p_convective_precipitation_6h`|% (0..100)|Probability: Occurrence of convective precipitation within the last 6 hours|wwC6|
|`p_thunderstorms_6h`|% (0..100)|Probability: Occurrence of thunderstorms within the last 6 hours|wwT6|
|`p_precipitation_6h`|% (0..100)|Probability: Occurrence of precipitation within the last 6 hours|wwP6|
|`p_liquid_precipitation_6h`|% (0..100)|Probability: Occurrence of liquid precipitation within the last 6 hours|wwL6|
|`p_freezing_rain_6h`|% (0..100)|Probability: Occurrence of freezing rain within the last 6 hours|wwF6|
|`p_solid_precipitation_6h`|% (0..100)|Probability: Occurrence of solid precipitation within the last 6 hours|wwS6|
|`p_drizzle_6h`|% (0..100)|Probability: Occurrence of drizzle within the last 6 hours|wwZ6|
|`p_fog_24h`|% (0..100)|Probability: Occurrence of fog within the last 24 hours|wwMd|
|`p_gusts_25kn_6h`|% (0..100)|Probability: Occurrence of gusts >= 25kn within the last 6 hours |FX625|
|`p_gusts_40kn_6h`|% (0..100)|Probability: Occurrence of gusts >= 40kn within the last 6 hours |FX640|
|`p_gusts_55kn_6h`|% (0..100)|Probability: Occurrence of gusts >= 55kn within the last 6 hours |FX655|
|`p_straightform_precipitation_12h`|% (0..100)|Probability: Occurrence of stratiform precipitation within the last 12 hours|wwDh|
|`p_convective_precipitation_12h`|% (0..100)|Probability: Occurrence of convective precipitation within the last 12 hours|wwCh|
|`p_thunderstorms_12h`|% (0..100)|Probability: Occurrence of thunderstorms within the last 12 hours|wwTh|
|`p_precipitation_12h`|% (0..100)|Probability: Occurrence of precipitation within the last 12 hours|wwPh|
|`p_liquid_precipitation_12h`|% (0..100)|Probability: Occurrence of liquid precipitation within the last 12 hours|wwLh|
|`p_freezing_rain_12h`|% (0..100)|Probability: Occurrence of freezing rain within the last 12 hours|wwFh|
|`p_solid_precipitation_12h`|% (0..100)|Probability: Occurrence of solid precipitation within the last 12 hours|wwSh|
|`p_drizzle_12h`|% (0..100)|Probability: Occurrence of drizzle within the last 12 hours|wwZh|
|`p_precipitation_1mm_6h`|% (0..100)|Probability of precipitation > 1.0 mm during the last 6 hours|R610|
|`precipitation_12h`|kg / m2|Total precipitation during the last 12 hours|RRh|
|`precipitation_12h_significant_weather`|kg / m2|Total precipitation during the last 12 hours consistent with significant weather|RRhc|
|`significant_weather_3h`|- (0..95)|Significant Weather of the last 3 hours|ww3|
|`liquid_precipitation_1h_significant_weather`|kg / m2|Total liquid precipitation during the last hour consistent with significant weather|RRL1c|
|`p_precipitation_00_24h`|% (0..100)|Probability of precipitation > 0.0 mm during the last 24 hours|Rd00|
|`p_precipitation_1mm_24h`|% (0..100)|Probability of precipitation > 1.0 mm during the last 24 hours|Rd10|
|`precipitation_24h`|kg / m2|Total precipitation during the last 24 hours|RRd|
|`precipitation_24h_significant_weather`|kg / m2|Total precipitation during the last 24 hours consistent with significant weather|RRdc|
|`cloud_cover_low_mid_7km`|% (0..100)|Cloud cover low and mid level clouds below 7000 m|Nlm|
|`p_precipitation_24h`|% (0..100)|Probability: Occurrence of any precipitation within the last 24 hours|wwPd|
|`cloud_base_convective_clouds`|m|Cloud base of convective clouds|H_BsC|
|`p_thunderstorms_24h`|% (0..100)|Probability: Occurrence of thunderstorms within the last 24 hours|wwTd|
|`e_surface_pressure`|Pa|Absolute error surface pressure|E_PPP|
|`sunshine_duration_3h`|s|Sunshine duration during the last three hours|SunD3|
|`opt_significant_weather_1h`|- (0..95)|Optional significant weather (highest priority) during the last hour|WPc11|
|`opt_significant_weather_3h`|- (0..95)|Optional significant weather (highest priority) during the last 3 hours|WPc31|
|`opt_significant_weather_6h`|- (0..95)|Optional significant weather (highest priority) during the last 6 hours|WPc61|
|`opt_significant_weather_12h`|- (0..95)|Optional significant weather (highest priority) during the last 12 hours|WPch1|
|`opt_significant_weather_24h`|- (0..95)|Optional significant weather (highest priority) during the last 24 hours|WPcd1|
|`accumulated_snow_3h`|m|Accumulated new snow amount in 3 hours|Sa3|
|`accumulated_snow_6h`|m|Accumulated new snow amount in 6 hours (amount of 3h values)|Sa6|
|`accumulated_snow_12h`|m|Accumulated new snow amount in 12 hours (amount of 6h values)|Sah|
|`accumulated_snow_24h`|m|Accumulated new snow amount in 24 hours (amount of 12h values)|Sad|
|`p_snow_5cm_6h`|% (0..100)|Probability of > 5cm new snow amount in 6 hours|Sa605|
|`p_snow_10cm_6h`|% (0..100)|Probability of > 10cm new snow amount in 6 hours|Sa610|
|`p_snow_20cm_6h`|% (0..100)|Probability of > 20cm new snow amount in 6 hours|Sa620|
|`p_snow_5cm_12h`|% (0..100)|Probability of > 5cm new snow amount in 12 hours|Sah05|
|`p_snow_10cm_12h`|% (0..100)|Probability of > 10cm new snow amount in 12 hours|Sah10|
|`p_snow_30cm_12h`|% (0..100)|Probability of > 30cm new snow amount in 12 hours|Sah30|
|`p_snow_10cm_24h`|% (0..100)|Probability of > 10cm new snow amount in 24 hours|Sad10|
|`p_snow_30cm_24h`|% (0..100)|Probability of > 30cm new snow amount in 24 hours|Sad30|
|`p_snow_50cm_24h`|% (0..100)|Probability of > 50cm new snow amount in 24 hours|Sad50|
|`snow_depth`|m|Snow depth|SnCv|
</details>

## `GET /stations`

Returns all Mosmix stations. The response is valid for a long time (a few weeks probably).
This is static data and it shouldn't be requested constantly.
The `id` property is used to request the weather forecast.

### Response 

```typescript
type StationsResponse = MosmixStation[];

interface MosmixStation {
  id: string;
  icao: string | null;
  name: string;
  latitude: number;
  longitude: number;
  elevation: number;
}
```

## `GET /report/{station}`

Returns the report for a given station (by its id). The values are one day old and updated every hour.
Like in the forecast, some values may not be present in some record (in this case, they're not in the object at all).

### Response 

```typescript
type Units = {[P in Properties]: string} // see table below

interface StationReport {
   units: Units;
   data: Array<{
        [P in Properties]?: number;
   } & { timestamp: timestamp_ms}>;
}
```

<details>
<summary>Properties</summary>

### Properties
| Property | Unit | 
|----------|------|
|`past_weather_1`|CODE_TABLE|
|`global_radiation_last_hour`|W/m2|
|`dry_bulb_temperature_at_2_meter_above_ground`|°C|
|`depth_of_new_snow`|cm|
|`maximum_wind_speed_last_hour`|km/h|
|`precipitation_last_12_hours`|mm|
|`global_radiation_past_24_hours`|W/m2|
|`minimum_temperature_last_12_hours_5_cm_above_ground`|Grad C|
|`mean_wind_direction_during_last_10 min_at_10_meters_above_ground`|°|
|`minimum_of_temperature_at_5_cm_above_ground_for_previous_day`|°C|
|`past_weather_2`|CODE_TABLE|
|`precipitation_amount_last_3_hours`|mm|
|`minimum_temperature_last_12_hours_2_meters_above_ground`|°C|
|`daily_mean_of_temperature_previous_day`|°C|
|`maximum_wind_speed_as_10_minutes_mean_during_last_hour`|km/h|
|`maximum_temperature_last_12_hours_2_meters_above_ground`|°C|
|`maximum_wind_speed_for_previous_day`|km/h|
|`horizontal_visibility`|km|
|`minimum_of_temperature_for_previous_day`|°C|
|`precipitation_amount_last_24_hours`|mm|
|`diffuse_solar_radiation_last_hour`|W/m2|
|`direct_solar_radiation_last_hour`|W/m2|
|`present_weather`|CODE_TABLE|
|`total_time_of_sunshine_past_day`|h|
|`maximum_of_temperature_for_previous_day`|°C|
|`total_time_of_sunshine_during_last_hour`|min|
|`maximum_of_10_minutes_mean_of_wind_speed_for_previous_day`|km/h|
|`direct_solar_radiation_last_24_hours`|W/m2|
|`precipitation_amount_last_6_hours`|mm|
|`total_snow_depth`|cm|
|`mean_wind_speed_during last_10_min_at_10_meters_above_ground`|km/h|
|`cloud_cover_total`|%|
|`precipitation_amount_last_hour`|mm|
|`evaporation/evapotranspiration_last_24_hours`|mm|
|`height_of_base_of_lowest_cloud_above_station`|m|
|`dew_point_temperature_at_2_meter_above_ground`|°C|
|`maximum_wind_speed_during_last_6_hours`|km/h|
|`pressure_reduced_to_mean_sea_level`|hPa|
|`temperature_at_5_cm_above_ground`|°C|
|`sea/water_temperature`|°C|
|`relative_humidity`|%|
</details>
