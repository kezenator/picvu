use std::collections::HashSet;
use curl::easy::Easy;
use serde::Deserialize;
use url::Url;

#[derive(Debug)]
pub struct ReverseGeocode
{
    pub address: String,
    pub names: HashSet<String>,
}

#[derive(Debug)]
pub struct GeocodeError(pub String);

pub fn reverse_geocode(api_key: &str, latitude: f64, longitude: f64) -> Result<ReverseGeocode, GeocodeError>
{
    let mut url : Url = "https://maps.googleapis.com/maps/api/geocode/json".parse().expect("Can't decode hard-coded URL");
    url.query_pairs_mut().append_pair("latlng", &format!("{},{}", latitude, longitude));
    url.query_pairs_mut().append_pair("key", api_key);

    let mut data = Vec::new();
    let mut handle = Easy::new();
    handle.url(&url.to_string())?;

    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        })?;
        transfer.perform()?;
    }

    let body = serde_json::from_slice::<JsonResponse>(&data)?;

    if body.status != "OK"
    {
        return Err(GeocodeError::new(format!("Bad response status: {:?}, msg={:?}", body.status, body.error_message.unwrap_or_default())));
    }

    let results = body.results
        .ok_or(GeocodeError::new("Bad response: missing results".to_owned()))?;

    // Ensure there is at least one result, and use
    // the first result as the address

    let address = results
        .iter()
        .nth(0)
        .ok_or(GeocodeError::new("Bad response: empty results".to_owned()))?
        .formatted_address
        .clone();

    // First, try and find the country

    let mut country = String::new();

    for r in results.iter()
    {
        if types_contains(&r.types, "country")
        {
            country = r.formatted_address.clone();
        }
    }

    // Now, collect all useful names

    let mut names = HashSet::new();

    for r in results.iter()
    {
        if types_wanted(&r.types)
            || types_contains(&r.types, "postal_code")
            || types_contains(&r.types, "street_address")
        {
            for e in r.address_components.iter()
            {
                if types_wanted(&e.types)
                {
                    if types_contains(&e.types, "administrative_area_level_2")
                        && (country == "Australia")
                    {
                        // Austrlian Admin level 2 are council names, with names like
                        // "Brisbane City" or "Cairns Regional" - we'll just use the short name.

                        names.insert(e.short_name.clone());
                    }
                    else
                    {
                        names.insert(e.long_name.clone());
                    }
                }
            }
        }
    }

    Ok(ReverseGeocode
    {
        address,
        names,
    })
}

fn types_wanted(types: &Vec<String>) -> bool
{
    return types_contains(types, "country")
        || types_contains(types, "administrative_area_level_1")
        || types_contains(types, "administrative_area_level_2")
        || types_contains(types, "administrative_area_level_3")
        || types_contains(types, "colloquial_area")
        || types_contains(types, "locality")
        || types_contains(types, "neighborhood")
        || types_contains(types, "natural_feature")
        || types_contains(types, "park")
        || types_contains(types, "point_of_interest");
}

fn types_contains(types: &Vec<String>, type_: &'static str) -> bool
{
    for t in types
    {
        if t == type_
        {
            return true;
        }
    }
    return false;
}

impl GeocodeError
{
    fn new(s: String) -> Self
    {
        GeocodeError(s)
    }
}

impl From<curl::Error> for GeocodeError
{
    fn from(source: curl::Error) -> Self
    {
        GeocodeError::new(format!("HTTP error: {:?}", source))
    }
}

impl From<serde_json::Error> for GeocodeError
{
    fn from(source: serde_json::Error) -> Self
    {
        GeocodeError::new(format!("Response decode error: {:?}", source))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct JsonResponse
{
    status: String,
    error_message: Option<String>,
    results: Option<Vec<ResultEntry>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ResultEntry
{
    address_components: Vec<AddressComponent>,
    formatted_address: String,
    geometry: Geometry,
    place_id: String,
    types: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AddressComponent
{
    long_name: String,
    short_name: String,
    types: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Geometry
{
    location: Location,
    location_type: String,
    viewport: Viewport,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Viewport
{
    northeast: Location,
    southwest: Location,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Location
{
    lat: f64,
    lng: f64,
}
