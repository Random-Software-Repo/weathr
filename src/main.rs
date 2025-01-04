extern crate printwrap;
use log::*;
use std::{env,fs,process,path::Path};
use minreq;
use serde_json::{Value};
use terminal_size::{Width, Height, terminal_size};
//use chrono::prelude::DateTime;
//use chrono::{DateTime,Weekday,Local,LocalResult,Datelike,TimeDelta,Duration,NaiveDateTime,offset::TimeZone};
use chrono::{DateTime,Local,TimeDelta};

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
	printwrap::print_wrap(10,30,"--purge                       Will delete all cached files for weathr. When next running weathr you will have to provide a location with the \"-l\" option.");
	printwrap::print_wrap(10,30,"-v | -vv                      Print verbose or very verbose information during the operation of weathr.");
	printwrap::print_wrap(10,30,"-w | -ww                      Prints output in 30 or 40 character columns.");
	printwrap::print_wrap(10,30,"                              By default, weathr prints forecast infomation in 20 character wide columns. It will print as many columns as your terminal window can completely show. Each column will correspond to a half-day forecast (one each for day and night). The wider your terminal window is, the more forecast columns you will see.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"The NWS API requires a location in decimal degrees of latitude and longitude. Neither weathr nor the NWS provide any mechanism to convert a \"normal\" street address or location to latitude and longitude. You can fairly easily get these values, however, with easily accessible services. Google Earth will show the latitude and longitude of the cursor in the lower right corner of the screen. Both Google Maps and Open Street Maps will show the lat/long of the center of the map in the URL / Address Bar after zooming in at least a little.");
	printwrap::print_wrap(5,0,"");
	printwrap::print_wrap(5,0,"Weathr makes several calls to the NWS API. The responses to these calls will be cached in ~/.config/weathr. Each response will have it's own expiration date provided in the response headers from the API call. Weathr will always abide by these suggestions. While weathr caches files, expired files will be deleted automatically for the current location only. If you use weathr with multiple locations, cached reponses for the other locations will not get automatically purged. See --purge above.");
	
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

fn purge_config()
{
	// use println! not logging as this will be called before logging is setup.
	let config_dir = get_config_dir();
	let config_dir_path = Path::new(config_dir.as_str());
	if config_dir_path.exists()
	{
		println!("Purging config directory: \"{}\"", config_dir);
		match fs::remove_dir_all(config_dir_path)
		{
			Ok(o)=> o,
			Err(e)=>{println!("Error purging config directory: \"{}\":{}",config_dir,e)},
		};
	}
	else
	{
		println!("Can't purge config directory \"{}\" as it doesn't exist.", config_dir);
	}
}

fn get_config_file_name() -> String
{
	let dir = get_config_dir();
	let file = "properties.json";
	let full_path = format!("{}/{}",dir,file);
	return full_path
}

fn cache_response(url:&str, expires:&str, body:&str)->bool
{
	// the cache responses will be saved in:
	//			$HOME/.config/weathr/<URL>/<EXPIRATION DATE>

	let config_dir = get_config_dir();
	let config_dir_path = Path::new(config_dir.as_str());
	if !config_dir_path.exists()
	{
		match fs::create_dir(config_dir_path)
		{
			Ok(o)=> o,
			Err(e)=>{error!("Error creating \"{}\":{}",config_dir_path.display(),e);return false},
		};
	}
	// the only character in the URLs that is not file-system safe (at least for unix-ish
	// file systems) is the '/'. we'll replace that with th unicode visually similar 
	// character '╱' so that we can simply use the URL as the directory in which to cache
	// the response. This make it easy to find here, and when browsing those directories.
	let fs_safe_url = url.replace("/","╱");
	let cache_dir = format!("{}/{}",config_dir,fs_safe_url);
	let cache_dir_path = Path::new(cache_dir.as_str());
	if !cache_dir_path.exists()
	{
		match fs::create_dir(cache_dir_path)
		{
			Ok(o)=> o,
			Err(e)=>{error!("Error creating \"{}\":{}",cache_dir_path.display(),e);return false},
		};
	}
	// ok, by this point we've created $HOME/.config/weathr/URL
	// now we write body into a file with the expiration as the file name
	if expires != ""
	{
		let file_name = format!("{}/{}",cache_dir, expires);
		match fs::write(file_name.as_str(), body)
		{
			Ok(o)=>o,
			Err(e)=>{error!("Error writing \"{}\":{}",file_name.as_str(),e);return false},
		}
	}
	return true;
}

fn get_cached_response(url:&str) -> String
{
	// Inspects the cached responses, and if one matches the url specified
	// and has not expried, it will be read and returned.
	// If and cached files matching the url specified exist but have expired,
	// they will be deleted.

	let mut data = String::from("");
	let config_dir = get_config_dir();
	let config_dir_path = Path::new(config_dir.as_str());
	if config_dir_path.exists()
	{
		let fs_safe_url = url.replace("/","╱");
		let cache_dir = format!("{}/{}",config_dir,fs_safe_url);
		let cache_dir_path = Path::new(cache_dir.as_str());
		if cache_dir_path.exists()
		{
			// iterate over files in this directory.
			// if any exist with a file name in the past, expired, delete them
			// if any file exists with a file name of now or in the future, read it and return that data.
			let now = Local::now();
			let paths = match fs::read_dir(cache_dir_path)
						{
							Ok(paths)=>paths,
							Err(e)=>{error!("Error reading \"{}\":{}",cache_dir_path.display(),e);return data},
						};
			for path in paths
			{
				let p1 = match path
						{
							Ok(p1)=>p1,
							Err(e)=>{error!("Error reading path {}",e); return data},
						};
				let file_name = String::from(match p1.file_name().to_str(){Some(s)=>s,None=>""});
				// convert file_name to a date
				let file_date = match DateTime::parse_from_rfc2822(file_name.as_str())
								{
									Ok(f)=>f,
									Err(e)=>{error!("error parsing date \"{}\":{}",file_name,e);return data},
								};
				//match SystemTime::now().duration_since(file_date) 
				let delta = now.signed_duration_since(file_date);
				/*****/
				// if duration is more than 0 seconds, 
				// the file_date is in the past and the 
				// file has expired, and should be deleted
				if delta > TimeDelta::seconds(0)
				{
					// The files are saved with the expiration date as the file name
					// If the current time is more than 0 seconds later than the file_date time
					// the cache has expired and should be deleted.
					match fs::remove_file(p1.path())
							{
								Err(e)=>error!("Error deleting expired file \"{}\":{}", p1.path().display(),e),
								Ok(o)=>o,
							};
				}
				else
				{
					// file is valid, read data and return that.
					data= match fs::read_to_string(p1.path())
								{
									Err(e)=>{error!("Error reading cached file \"{}\":{}",p1.path().display(),e);data},
									Ok(data)=>{debug!("Returning Cached Data.");data},
								};
				}
				debug!("****    Name: \"{}\"",file_name);
			}
		}
		else
		{
			debug!("Cache dir \"{}\" doesn't exist.", cache_dir_path.display());
		}
	}
	else
	{
		debug!("Config dir \"{}\" doesn't exist.", config_dir_path.display());
	}
	return data;
}

fn call_nws_api(request_url:&str) -> String
{
	debug!("call_nws_aps \"{}\"", request_url);
	let cached = get_cached_response(request_url);
	if cached != ""
	{
		debug!("Call cached, using that data rather than requesting over http/s.");
		return cached;
	}
	else
	{
		debug!("Call Not Cached!");
		let o = match minreq::get(request_url)
				// nws api requires a user-agent header. doesn't matter what. anything will do, but is required.
				.with_header("User-Agent", "weathr-app")
				.send()
				{
					Err(e)=>{error!("Error making nws call:{}",e);process::exit(10)},
					Ok(o)=>o,
				};
		let expires = match o.headers.get("expires")
					{
						Some(expires)=>expires,
						None=> "", // should set to something like now+30 days ???
					};
		debug!("**** Expires : \"{}\"",expires);
		let s = match o.as_str()
			{
				Err(e)=>{error!("Error converting output to String: {}",e);process::exit(11)},
				Ok(s)=>s,
			};
		trace!("call_nws_output:\"{}\"",s);
		cache_response(request_url, expires, s);
		return String::from(s)
	}
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

fn get_location(pjson: &serde_json::Value, key: &str)->String
{
	let mut location="";
	//			let pname = match p["name"].as_str() {None=>"",Some(s)=>s};
	debug!("get_location \"{}\"",key);
	debug!("get_location \"{}\"",*pjson);
	let rl = &pjson["relativeLocation"];
	if *rl != Value::Null
	{
			debug!("got relativeLocation:\n{}", rl);
			let p2: &Value = &rl["properties"];
			if *p2 != Value::Null
			{
				debug!("got second properties:\n{}", p2);
				location = match p2[key].as_str()
					{
						None=>"",
						Some(c)=>c,
					};
			}
			else
			{
				debug!("failed to get second properties.");
			}
	}
	else
	{
		debug!("failed to get relativeLocation.");
	}
	return String::from(location)
}

fn get_city(pjson: &serde_json::Value)->String
{
	return get_location(pjson, "city")
}
fn get_state(pjson: &serde_json::Value)->String
{
	return get_location(pjson, "state")
}

fn print_location(pjson: serde_json::Value)
{
	let location = format!("{}, {}", get_city(&pjson), get_state(&pjson));
	println!("{}",location);
}

fn print_forecast( fjson : serde_json::Value, column_width:usize)
{
	//column_width = 20 as usize;
	let columns = get_terminal_width() / column_width as usize;
	// these vars should definitely be some sort of string buffer, but this works for now
	let mut name=String::from("");
	let mut short=String::from("");
	let mut short_line2 = String::from("");
	let mut short_line2_bool=false;
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

			let strings = printwrap::split(column_width, pshortforecast);
			let mut linec=0;
			for line in strings
			{
				if linec==0
				{
					short = format!("{}{:^column_width$}",short,line);
				}
				else if linec==1
				{
					short_line2_bool = true;
					short_line2 = format!("{}{:^column_width$}", short_line2,line);
				}
				// so, right now, this can only handle two lines of short-forecast.
				// text longer than will fit will be omitted from the output. using 
				// the -w command line option (wide, or -ww very wide) will help.
				linec=linec+1;
			}
			if linec==1
			{
				// was only one element, so we need to pad short2
				short_line2 = format!("{}{:^column_width$}", short_line2,"");
			}
			let ttemp=format!("{} {}°{}",temp_label,ptemperature,ptemperatureunit);
			temp = format!("{}{:^column_width$}",temp,ttemp);
			//rain = format!("{}{:^column_width}",rain,p["name"]);
			let wwind=format!("{} {}",pwindspeed,pwinddirection);
			wind = format!("{}{:^column_width$}",wind,wwind);
		}
	}
	println!("{}",name);
	println!("{}",short);
	if short_line2_bool
	{
		// only print line two if we need to.
		println!("{}",short_line2)
	}
	println!("{}",temp);
	println!("{}",wind);
}


fn load_forecast(url:&str, file:&str) -> serde_json::Value
{
	let forecast;
	if file != ""
	{
		forecast = match fs::read_to_string(file)
					{
						Err(e)=>{error!("Error loading forecast from file \"{}\":{}",file,e); process::exit(14)},
						Ok(f)=>f,
					};
	}
	else
	{
		forecast = call_nws_api(url);
	}
	let forecast_json: serde_json::Value = match serde_json::from_str(forecast.as_str())
	{
		Ok(json)=> json,
		Err(e)=>{error!("Error parsing forecast json:{}", e);process::exit(1)},
	};
	return forecast_json;
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
		config = call_nws_api(url);
	
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

fn make_config_url(latlong:&str)->String
{
	let url:String;
	if latlong == ""
	{
		url = String::from("");
	}
	else
	{
		url = format!("https://api.weather.gov/points/{}",latlong);
	}
	return url
}

fn make_latlong(latitude: &str, longitude: &str, latlong: &str) -> String
{
	let mut ll = String::from(latlong);
	debug!("latitude:\"{}\"",latitude);
	debug!("longitude:\"{}\"",longitude);
	debug!("latlong:\"{}\"",latlong);
	if (latlong == "") && ((latitude!="") || (longitude != ""))
	{
		if latitude==""
		{
			error!("Latitude not set but Longitude is set. Both Latitude and Longtiude are required.");
			process::exit(30);
		}
		if longitude == ""
		{
			error!("Longitude not set but Latitude is set. Both Latitude and Longtiude are required.");
			process::exit(30);
		}
		ll = format!("{},{}",latitude,longitude);
	}
	return ll
}

fn get_property(prop:&serde_json::Value, key:&str)->String
{
	if prop[key].is_string()
	{
		let val = match prop[key].as_str()
				{
					None=> {error!("Error getting \"{}\" from properties as a string.",key);process::exit(25)},
					Some(f)=>String::from(f),
				};
		return val;
	}
	else if prop[key].is_number()
	{
		let val = match prop[key].as_number()
				{
					None=> {error!("Error getting \"{}\" from properties as a number.",key);process::exit(25)},
					Some(f)=>f,
				};
		return format!("{}",val);
	}
	return String::from("");
}

fn get_features_property(prop:&serde_json::Value, key:&str) -> String
{
	let empty = String::from("");
	//"features" "0" "id"

	//let features:&Value = &prop["features"];

	let s1 = &prop["features"][0];
	if *s1 != Value::Null
	{
		//id = match s1[key].as_str() {None=>"",Some(s)=>s};
		if s1[key].is_string()
		{
			let val = match s1[key].as_str()
					{
						None=> {error!("Error getting \"{}\" from properties as a string.",key);process::exit(25)},
						Some(f)=>String::from(f),
					};
			return val;
		}
		else if s1[key].is_number()
		{
			let val = match s1[key].as_number()
					{
						None=> {error!("Error getting \"{}\" from properties as a number.",key);process::exit(25)},
						Some(f)=>f,
					};
			return format!("{}",val);
		}
	}
	return empty;
}

fn get_features_property_value(prop:&serde_json::Value, index:usize, key:&str, key2:&str) -> String
{
	let empty = String::from("");
	//"features" "0" "id"

	//let features:&Value = &prop["features"];

	let feature = &prop["features"][index];
	if *feature != Value::Null
	{
		let fproperties = &feature["properties"];
		if *fproperties == Value::Null
						{
							error!("Error getting features/properties");
							process::exit(23);
						}
		let s2 = &fproperties[key];
		if *s2 != Value::Null
		{
			debug!("s2:\n{}",s2);
			//id = match s1[key].as_str() {None=>"",Some(s)=>s};
			if s2[key2].is_string()
			{
				let val = match s2[key2].as_str()
						{
							None=> {error!("Error getting \"{}\" from properties as a string.",key2);process::exit(25)},
							Some(f)=>String::from(f),
						};
				return val;
			}
			else if s2[key2].is_number()
			{
				let val = match s2[key2].as_number()
						{
							None=> {error!("Error getting \"{}\" from properties as a number.",key2);process::exit(25)},
							Some(f)=>f,
						};
				return format!("{}",val);
			}
		}
		else
		{
			debug!("s2 \"{}\" is null",key);
		}
	}
	else
	{
		debug!("s1 is null");
	}
	return empty;
}

fn print_current_temperature(x:&str, y:&str, output_unit:&str)
{
	debug!("GridX:{}, GridY:{}", x, y);
	let stations_url=format!("https://api.weather.gov/gridpoints/TWC/{},{}/stations",x,y);
	let stations_string=call_nws_api(stations_url.as_str());
	let stations_json: serde_json::Value = match serde_json::from_str(stations_string.as_str())
			{
				Ok(json)=> json,
				Err(e)=>{error!("Error parsing stations json:{}", e);process::exit(1)},
			};
	let station_id = get_features_property(&stations_json,"id");
	if station_id !=""
	{
		let observations_url=format!("{}/observations",station_id);
		debug!("Observations url:\"{}\"",observations_url);

		let observations_string = call_nws_api(observations_url.as_str());
		let observations_json: serde_json::Value = match serde_json::from_str(observations_string.as_str())
				{
					Ok(json)=> json,
					Err(e)=>{error!("Error parsing observations json:{}", e);process::exit(1)},
				};
		//get_features_property_value_all(&observations_json,"temperature", "value");
		let mut index=0 as usize;
		let mut temp = get_features_property_value(&observations_json,index,"temperature", "value");
		while (index < 10) && (temp == "")
		{
			index = index +1;
			temp = get_features_property_value(&observations_json,index,"temperature", "value");
		}
		let mut sunit = get_features_property_value(&observations_json,index, "temperature", "unitCode");
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
		let temperature = format!("{}°{}", temp, sunit);
		println!("{:^20}",temperature);
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
	let mut forecast_file="";
	let mut skip_argument=false;
	let mut latitude="";
	let mut longitude="";
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
				"--forecastfile" =>
					{
						if (i+1) < end
						{
							forecast_file = args[i+1].as_str();
							skip_argument = true;
						}
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
				"--lat" =>
					{
						if (i+1) < end
						{
							latitude=&args[i+1];
							skip_argument = true;
						}
					}
				"--long" =>
					{
						if (i+1) < end
						{
							longitude=&args[i+1];
							skip_argument = true;
						}
					}
				"--purge" =>
					{
						purge_config();
						process::exit(0);
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

	latlong = make_latlong(latitude, longitude, latlong.as_str());
	config_url = make_config_url(latlong.as_str());
	debug!("config url:\"{}\"",config_url);

	let properties_json = load_config(config_url.as_str(), config_file.as_str());
	let gridx = get_property(&properties_json, "gridX");
	let gridy = get_property(&properties_json, "gridY");
	let forecast_url = get_property(&properties_json, "forecast");


	let forecast_json = load_forecast(forecast_url.as_str(),forecast_file);
	print_location(properties_json);
	print_current_temperature(gridx.as_str(), gridy.as_str(), "F");
	print_forecast(forecast_json,column_width);

}
