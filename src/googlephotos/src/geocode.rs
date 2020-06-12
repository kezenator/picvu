use std::collections::{BTreeMap, HashMap};
use curl::easy::Easy;
use serde::Deserialize;
use url::Url;

#[derive(Debug)]
pub struct ReverseGeocode
{
    pub address: String,
    pub names: BTreeMap<GeocodeNameType, String>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GeocodeNameType
{
    Country,
    AdminAreaLevel1,
    AdminAreaLevel2,
    AdminAreaLevel3,
    AdminAreaLevel4,
    AdminAreaLevel5,
    Colloquial,
    Locality,
    SubLocality,
    Neighbourhood,
    NaturalFeature,
    Park,
    PointOfInterest,
    Route,
}

#[derive(Debug)]
pub struct GeocodeError(String);

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

    println!("Data: {:?}", String::from_utf8_lossy(&data));

    let body = serde_json::from_slice::<JsonResponse>(&data)?;

    if body.status != "OK"
    {
        return Err(GeocodeError::new(format!("Bad response status: {:?}, msg={:?}", body.status, body.error_message.unwrap_or_default())));
    }

    let entry = body.results
        .ok_or(GeocodeError::new("Bad response: missing results".to_owned()))?
        .drain(..)
        .nth(0)
        .ok_or(GeocodeError::new("Bad response: zero results".to_owned()))?;

    let mut names = BTreeMap::new();

    for addr in entry.address_components
    {
        for type_ in addr.types
        {
            match type_.as_str()
            {
                "country" => { names.insert(GeocodeNameType::Country, addr.long_name.clone()); },
                "administrative_area_level_1" => { names.insert(GeocodeNameType::AdminAreaLevel1, addr.long_name.clone()); },
                "administrative_area_level_2" => { names.insert(GeocodeNameType::AdminAreaLevel2, addr.long_name.clone()); },
                "administrative_area_level_3" => { names.insert(GeocodeNameType::AdminAreaLevel3, addr.long_name.clone()); },
                "administrative_area_level_4" => { names.insert(GeocodeNameType::AdminAreaLevel4, addr.long_name.clone()); },
                "administrative_area_level_5" => { names.insert(GeocodeNameType::AdminAreaLevel5, addr.long_name.clone()); },
                "colloquial_area" => { names.insert(GeocodeNameType::Colloquial, addr.long_name.clone()); },
                "locality" => { names.insert(GeocodeNameType::Locality, addr.long_name.clone()); },
                "sublocality" => { names.insert(GeocodeNameType::SubLocality, addr.long_name.clone()); },
                "neighborhood" => { names.insert(GeocodeNameType::Neighbourhood, addr.long_name.clone()); },
                "natural_feature" => { names.insert(GeocodeNameType::NaturalFeature, addr.long_name.clone()); },
                "park" => { names.insert(GeocodeNameType::Park, addr.long_name.clone()); },
                "point_of_interest" => { names.insert(GeocodeNameType::PointOfInterest, addr.long_name.clone()); },
                _ => {},
            }
        }
    }

    Ok(ReverseGeocode
    {
        address: entry.formatted_address,
        names: names,
    })
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct JsonResponse
{
    status: String,
    error_message: Option<String>,
    results: Option<Vec<ResultEntry>>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ResultEntry
{
    address_components: Vec<AddressComponent>,
    formatted_address: String,
    geometry: Geometry,
    place_id: String,
    types: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AddressComponent
{
    long_name: String,
    short_name: String,
    types: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Geometry
{
    location: Location,
    location_type: String,
    viewport: HashMap<String, Location>
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Location
{
    lat: f64,
    lng: f64,
}
