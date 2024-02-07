use candid::types::Type;
use ic_cdk::api;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpMethod,HttpResponse,
};
use serde_json::Value;
use candid::{CandidType, Decode, Deserialize, Encode, Nat};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell}; 

#[derive(CandidType, Deserialize,Debug)]
enum Activities {
    stayhome,
    library,
    park,
    shopping,
}
#[derive(CandidType, Deserialize)]
struct Person{
    name:String,
    lastname:String,
    age:u32,
}
#[derive(CandidType, Deserialize)]
enum PersonErrors {
    SmallAge,
}
impl Storable for Person {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
type Memory = VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 = 100;

impl BoundedStorable for Person {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE; 
    const IS_FIXED_SIZE: bool = false;
}
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static Person_Map: RefCell<StableBTreeMap<u64, Person, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|p| p.borrow().get(MemoryId::new(1))), 
        )
    );
}


#[ic_cdk::update]
fn create_person(name:String,lastname:String,age:u32)-> Result<(),PersonErrors>{
    Person_Map.with(|p| {
        let mut person_map = p.borrow_mut();
        for (_,person) in person_map.iter()  {
            if  age<10 {
                return Err(PersonErrors::SmallAge);
            } 
        };
        let person = Person{
            name,
            lastname,
            age,
           
        };
        let new_person_id=person_map.len();
        person_map.insert(new_person_id,person);  

        Ok(())
    })
}


#[ic_cdk::update]
async fn suggest_activity() -> Activities {
    let temperature = get_events_from_api().await;
    if temperature >=10.0 && temperature<=25.0 {
        Activities::park
    }
    else if temperature<=10.0 && temperature >=0.0 {
        Activities::library
    }
    else if temperature>0.0 && temperature<=-10.0 {
        Activities::shopping
    }
    else {
        Activities::stayhome
    }
}


#[ic_cdk::update]
async fn get_events_from_api() -> f64 {
    // Setup the URL for the HTTP GET request

    let api_key = "85418a7c7fc4445b8db2a38f114c45b4";
    let city = "Istanbul";
    let url = format!("https://api.weatherbit.io/v2.0/current?&city={}&key={}", city, api_key);
    // Prepare headers for the system http_request call
    let request_headers = vec![];

    // Setup the HTTP request arguments
    let request = CanisterHttpRequestArgument {
        url,
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers:request_headers ,
    };
    let response = http_request(request, 1_603_096_000).await.unwrap().0;
    // Parse JSON response
    let json_response: Value= serde_json::from_slice(&response.body).expect("Failed to parse JSON response.");

    // Extract the 'temp' value from the JSON
    let temp = json_response["data"][0]["app_temp"].as_f64().expect("Failed to extract 'temp' value from JSON");

    temp
}









