extern crate printwrap;
extern crate nws;
use log::*;
use std::{env,fs,process,path::Path};
use serde_json::{Value};
use terminal_size::{Width, Height, terminal_size};
use chrono::{DateTime,Local};

fn usage()
{
	printwrap::print_wrap(5,0,"weathr!");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"Weathr queries the U.S. National Weather Service public API for current conditions and 7 day forecast for any locations serviced by the U.S. NWS.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"Usage:");
	printwrap::print_wrap(10,0,"weathr [options]");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"Options:");
	printwrap::print_wrap(10,30,"-h | --help                   This usage information.");
	printwrap::print_wrap(10,30,"-l | --latlong <lat>,<long>   Latitude and Longitude in decimal format of the location you want the weather for. After weathr has been run once with valid lat/long, it will cache that information and subsequent calls to weathr can omit the latitude and longitude but will continue to display (and update) weather for that location. To change location, simply provide a new lat/long.");
	printwrap::print_wrap(10,30,"-v | -vv                      Print verbose or very verbose information during the operation of weathr.");
	printwrap::print_wrap(10,30,"-w | -ww                      Prints output in 30 or 40 character columns.");
	printwrap::print_wrap(10,30,"                              By default, weathr prints forecast infomation in 20 character wide columns. It will print as many columns as your terminal window can completely show. Each column will correspond to a half-day forecast (one each for day and night). The wider your terminal window is, the more forecast columns you will see.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"The NWS API requires a location in decimal degrees of latitude and longitude. Neither weathr nor the NWS provide any mechanism to convert a \"normal\" street address or location to latitude and longitude. You can fairly easily get these values, however, with easily accessible services. Google Earth will show the latitude and longitude of the cursor in the lower right corner of the screen. Both Google Maps and Open Street Maps will show the lat/long of the center of the map in the URL / Address Bar after zooming in at least a little.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"Weathr makes several calls to the NWS API. The responses to these calls will be cached in ~/.config/weathr. Each response will have it's own expiration date provided in the response headers from the API call. Weathr will always abide by these suggestions. While weathr caches files, expired files will be deleted automatically for the current location only. If you use weathr with multiple locations, cached reponses for the other locations will not get automatically purged. See --purge above.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"Weathr gets current conditions from the nearest observation station to the location provided. However, not all locations serviced by the NWS seem to have local observation stations. For these locations, no current conditions will be printed.");
	
	process::exit(2);
}

fn get_var(key:&str) -> String
{
	let empty = String::from("");
	let val = match env::var(key)
				{
					Err(e) => {error!("\"{}\"", e);empty},
					Ok(val) => val,
				};
	return val;
}

fn get_config_dir() -> String
{
	// Will return $HOME/.config/weathr and create the sub-directories of
	// ".config/weathr" if they don't exist. Otherwise will return "/dev/null"

	let dev_null = String::from("/dev/null");
	let home = get_var("HOME").to_owned();
	if home == ""
	{
		return dev_null;
	}
	let config_base_dir=format!("{}/.config",home);
	let config_base_dir_path = Path::new(config_base_dir.as_str());
	if !config_base_dir_path.exists()
	{
		match fs::create_dir(config_base_dir_path)
		{
			Ok(o)=> o,
			Err(e)=>{error!("Error creating \"{}\":{}",config_base_dir,e);return dev_null},
		};
	}

	let config_dir = format!("{}/weathr",config_base_dir);
	let config_dir_path = Path::new(config_dir.as_str());
	if !config_dir_path.exists()
	{
		match fs::create_dir(config_dir_path)
		{
			Ok(o)=> o,
			Err(e)=>{error!("Error creating \"{}\":{}",config_dir,e);return dev_null},
		};
	}
	return config_dir
}


fn get_config_file_name() -> String
{
	let dir = get_config_dir();
	let file = "properties.json";
	let full_path = format!("{}/{}",dir,file);
	return full_path
}

fn get_terminal_width() -> usize
{
	let x;

	let size = terminal_size();
	if let Some((Width(w), Height(_h))) = size
	{
		x = w as usize;
	}
	else
	{
		x = 100 as usize;
	}
	return x
}

fn save_config(data:&str, file:&str) -> bool
{
	let home = get_var("HOME").to_owned();
	//let mut sep = "";
	if home != ""
	{
		//sep = "/"
		let config_dir = format!("{}/.config",home);
		let weathr_dir = format!("{}/.config/weathr",home);
		let config_file = format!("{}/{}",weathr_dir,file);
		let config_dir_path = Path::new(config_dir.as_str());
		let weathr_dir_path = Path::new(weathr_dir.as_str());
		//let config_file_path = Path::new(config_file.as_str());
		if !config_dir_path.exists()
		{
			match fs::create_dir(config_dir_path)
			{
				Ok(o)=> o,
				Err(e)=>{error!("Error creating \"{}\":{}",config_dir_path.display(),e);return false},
			};
		}
		if !weathr_dir_path.exists()
		{
			match fs::create_dir(weathr_dir_path)
			{
				Ok(o)=> o,
				Err(e)=>{error!("Error creating \"{}\":{}",weathr_dir_path.display(),e);return false},
			};
		}
		match fs::write(config_file.as_str(), data)
		{
			Ok(o)=>o,
			Err(e)=>{error!("Error writing \"{}\":{}",config_file.as_str(),e);return false},
		}
	}
	else
	{
		error!("no home directory.");
		return false;
	}
	return true
}

fn print_location(pjson: serde_json::Value)
{
	let location = format!("{}, {}", nws::nws::get_city(&pjson), nws::nws::get_state(&pjson));
	println!("{}",location);
}

fn print_forecast( fjson : serde_json::Value, column_width:usize)
{
	//column_width = 20 as usize;
	let columns = get_terminal_width() / column_width as usize;
	// these vars should definitely be some sort of string buffer, but this works for now
	let mut name=String::from("");
	const SHORT_LINE_MAX:usize = 6;
	let mut short_lines:[String;SHORT_LINE_MAX] = [String::from(""),String::from(""),String::from(""),String::from(""),String::from(""),String::from("")];
	let mut short_line_count:usize = 0;
	let mut temp=String::from("");
	//let mut rain=String::from("");
	let mut wind=String::from("");
	let properties: &Value = &fjson["properties"];
	//let p1 = &properties["periods"][0];
	for n in 0 .. columns
	{
		let p = &properties["periods"][n];
		if *p != Value::Null
		{
			let pname = match p["name"].as_str() {None=>"",Some(s)=>s};
			let pshortforecast = match p["shortForecast"].as_str() {None=>"",Some(s)=>s};
			let pisdaytime = match p["isDaytime"].as_bool() {None=>true,Some(s)=>s};
			let temp_label = if pisdaytime { "High near"} else {"Low near"};
			let ptemperature = &p["temperature"];// {None=>String::from(""),Some(s)=>format!("{}",s)};
			let ptemperatureunit = match p["temperatureUnit"].as_str() {None=>"",Some(s)=>s};
			let pwindspeed = match p["windSpeed"].as_str() {None=>"",Some(s)=>s};
			let pwinddirection = match p["windDirection"].as_str() {None=>"",Some(s)=>s};

			name = format!("{}{:^column_width$}",name,pname);

			let strings = printwrap::split(column_width-1, pshortforecast);
			let mut linec=0;
			for line in strings
			{
				if linec < SHORT_LINE_MAX
				{
					let short_line = format!("{}{:^column_width$}",short_lines[linec],line);
					short_lines[linec] = short_line;
					if short_line_count < (linec+1)
					{
						short_line_count = linec+1;
					}
				}
				linec=linec+1;
			}
			while linec < SHORT_LINE_MAX
			{
				// pads out all unused lines
				let short_line = format!("{}{:^column_width$}",short_lines[linec],"");
				short_lines[linec] = short_line;
				linec = linec + 1;
			}
			let ttemp=format!("{} {}°{}",temp_label,ptemperature,ptemperatureunit);
			temp = format!("{}{:^column_width$}",temp,ttemp);
			//rain = format!("{}{:^column_width}",rain,p["name"]);
			let wwind=format!("{} {}",pwindspeed,pwinddirection);
			wind = format!("{}{:^column_width$}",wind,wwind);
		}
	}
	println!("{}",name);

	for n in 0..short_line_count
	{
		println!("{}",short_lines[n]);
	}
	println!("{}",temp);
	println!("{}",wind);
}


fn load_config(url:&str, file:&str) -> serde_json::Value
{
	debug!("Load_config \"{}\",\"{}\"", url, file);
	let config:String;
	if url == ""
	{
		debug!("Loading Config from file: \"{}\"",file);
		config = match fs::read_to_string(file)
					{
						Ok(data)=>data,
						Err(e)=>{error!("Error reading config file \"{}\":{}\n\nIf this is the first time you've run weathr, you must supply the latitude and longitude of your location using the \"-l\" option. Run \"weathr -h\" for more instructions.",file, e);process::exit(1)},
					};
		//trace!("config:\n{}",config);
	}
	else
	{
		// load from url
		config = nws::nws::call_nws_api(url);
	
		if config != ""
		{
			// call_nws_api will cache the data from the api, but we will
			// need a "default" properties.json for usage when no lat/long
			// was provided on the command line which means we can't make the
			// nws call and must depend on cached results.
			save_config(config.as_str(), "properties.json");
		}
	}
	let json: serde_json::Value = match serde_json::from_str(config.as_str())
			{
				Ok(json)=> json,
				Err(e)=>{error!("Error parsing json:{}", e);process::exit(1)},
			};
	//trace!("json:\n{}",json);
	let properties: &serde_json::Value = &json["properties"];
	return properties.clone()
}

fn format_time(timestamp:&str)->String
{
	let error=String::from("");
	let time_in = match DateTime::parse_from_rfc3339(timestamp)
					{
						Ok(f)=>f,
						//Err(e)=>{error!("error parsing date \"{}\":{}",timestamp,e);return error},
						Err(_e)=>return error,
					};
	let time_local: DateTime<Local> = DateTime::from(time_in);
	let time_only = time_local.format("%l:%M%P");
	return format!("{}",time_only);
}

fn print_current_temperature(office:&str, x:&str, y:&str, output_unit:&str)
{
	debug!("GridX:{}, GridY:{}", x, y);
	let stations_url=format!("https://api.weather.gov/gridpoints/{}/{},{}/stations",office,x,y);
	let stations_string=nws::nws::call_nws_api(stations_url.as_str());
	let stations_json: serde_json::Value = match serde_json::from_str(stations_string.as_str())
			{
				Ok(json)=> json,
				Err(e)=>{error!("Error parsing stations json:{}", e);process::exit(1)},
			};
	//let station_id = get_features_property(&stations_json,0, "id");
	let station_id = nws::nws::get_features_properties_key(&stations_json,0, "stationIdentifier");
	let station_name = nws::nws::get_features_properties_key(&stations_json,0, "name");
	let station_url = nws::nws::get_features_key(&stations_json,0,"id");
	if station_id !=""
	{
		let observations_url=format!("{}/observations",station_url);
		debug!("Observations url:\"{}\"",observations_url);

		let observations_string = nws::nws::call_nws_api(observations_url.as_str());
		let observations_json: serde_json::Value = match serde_json::from_str(observations_string.as_str())
				{
					Ok(json)=> json,
					Err(e)=>{error!("Error parsing observations json:{}", e);process::exit(1)},
				};
		//get_features_property_value_all(&observations_json,"temperature", "value");
		let mut index=0 as usize;
		let mut temp = nws::nws::get_features_properties_value_key(&observations_json,index,"temperature", "value");
		let mut timestamp = nws::nws::get_features_properties_key(&observations_json,index,"timestamp");
		let mut time_representation = format_time(timestamp.as_str());
		while (index < 10) && (temp == "")
		{
			index = index +1;
			temp = nws::nws::get_features_properties_value_key(&observations_json,index,"temperature", "value");
			timestamp = nws::nws::get_features_properties_key(&observations_json,index,"timestamp");
			time_representation = format_time(timestamp.as_str());
		}
		if temp != ""
		{
			let mut sunit = nws::nws::get_features_properties_value_key(&observations_json,index, "temperature", "unitCode");
			//let p = unit.split("wmoUnit:deg");
			if let Some((_prefix, unit)) = sunit.split_once(":deg")
			{
				if output_unit != unit
				{
					let mut ftemp:f32 = match temp.parse() 
							{
								Err(_error) => -1000.0,//{error!("first parsing temperature \"{}\" is invalid:{}",temp,error);0.0},
								Ok(ftemp) => ftemp,
							};
					if (unit == "C") && (output_unit == "F")
					{
						ftemp = ((ftemp / 5.0) * 9.0) + 32.0;
						sunit=String::from("F");
					}
					else
					{
						ftemp = ((ftemp - 32.0) / 9.0) * 5.0;
						sunit=String::from("C");
					}
					if ftemp < -1000.0
					{
						temp = String::from("(missing)");
					}
					else
					{
						temp = format!("{:.0}",ftemp);
					}
				}
			}
			let temperature = format!("Most recent observation from {}({}) at {}: {}°{}", station_name, station_id, time_representation, temp, sunit);
			println!("{:^20}",temperature);
		}
		else
		{
			println!("No current weather observations.");
		}
	}
	else
	{
		println!("No current weather observations.");
	}
}

fn main() 
{
	let args: Vec<String> = env::args().collect();
	let start=1;
	let end=args.len();
	let mut verbose = log::Level::Info; // default log level of INFO
	let mut config_file=get_config_file_name();
	let config_url:String;
	let mut skip_argument=false;
	let mut latlong=String::from("");
	let mut column_width:usize=20;
	//let mut forecast=String::from("");
	//let mut config=String::from("");

	for i in start..end
	{
		if skip_argument
		{
			skip_argument = false;
		}
		else
		{
			match args[i].as_ref()
			{
				"-h" | "--help" =>
					{
					usage();
					}
				"-c"|"--configfile" =>
					{
						if (i+1) < end
						{
							config_file = String::from(args[i+1].as_str());
							skip_argument = true;
						}
					}
				"-l" | "--latlong" =>
					{
						if (i+1) < end
						{
							latlong=String::from(&args[i+1]);
							skip_argument = true;
						}
					}
				"-v" =>
					{
						verbose = log::Level::Debug;
					} 
				"-vv" =>
					{
						verbose = log::Level::Trace;
					}
				"-w" =>
					{
						column_width=30;
					}
				"-ww" =>
					{
						column_width=40;
					}
				_ =>
					{
						println!("Unknown argument \"{}\".",args[i]);
						usage();
					}

			}
		}
	}

	match stderrlog::new().module(module_path!()).verbosity(verbose).init()
	{
		Ok(l)=> l,
		Err(e) =>{println!("Failed to create stderr logger:{}",e)},
	}

	config_url = nws::nws::get_points_url(latlong.as_str());
	debug!("config url:\"{}\"",config_url);

	let properties_json = load_config(config_url.as_str(), config_file.as_str());
	let office = nws::nws::get_key(&properties_json,"gridId");
	let gridx = nws::nws::get_key(&properties_json, "gridX");
	let gridy = nws::nws::get_key(&properties_json, "gridY");
	let forecast_url = nws::nws::get_key(&properties_json, "forecast");


	//let forecast_json = nws::nws::load_forecast(forecast_url.as_str(),forecast_file);
	let forecast_json = nws::nws::load_forecast(forecast_url.as_str());
	print_location(properties_json);
	print_current_temperature(office.as_str(), gridx.as_str(), gridy.as_str(), "F");
	print_forecast(forecast_json,column_width);
	nws::nws::purge_config();
}
