use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, cell::RefCell};


type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct BasketRental {
    id: u64,
    nama_peminjam: String,
    tanggal_pinjam: u64,
    jam: u64,
    tipe_lapangan: char,
    status: String,
}

impl Storable for BasketRental {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for BasketRental {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static STORAGE: RefCell<StableBTreeMap<u64, BasketRental, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct BasketRentalPayload {
    nama_peminjam: String,
    tanggal_pinjam: u64,
    jam: u64,
    tipe_lapangan: char,
}

#[ic_cdk::query]
fn get_basket_rental(id: u64) -> Result<BasketRental, Error> {
    match _get_basket_rental(&id) {
        Some(rental) => Ok(rental),
        None => Err(Error::NotFound {
            msg: format!("a rental with id={} not found", id),
        }),
    }
}

#[ic_cdk::update]
fn add_basket_rental(payload: BasketRentalPayload) -> Option<BasketRental> {
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment id counter");

    let status = if payload.jam >= 17 {
        "lunas".to_string()
    } else {
        "belum".to_string()
    };

    let rental = BasketRental {
        id,
        nama_peminjam: payload.nama_peminjam,
        tanggal_pinjam: payload.tanggal_pinjam,
        jam: payload.jam,
        tipe_lapangan: payload.tipe_lapangan,
        status,
    };

    do_insert(&rental);
    Some(rental)
}

#[ic_cdk::update]
fn update_basket_rental(id: u64, payload: BasketRentalPayload) -> Result<BasketRental, Error> {
    match STORAGE.with(|service| service.borrow().get(&id)) {
        Some(mut rental) => {
            rental.nama_peminjam = payload.nama_peminjam;
            rental.tanggal_pinjam = payload.tanggal_pinjam;
            rental.jam = payload.jam;
            rental.tipe_lapangan = payload.tipe_lapangan;

            let status = if rental.jam >= 17 {
                "lunas".to_string()
            } else {
                "belum".to_string()
            };

            rental.status = status;
            do_insert(&rental);
            Ok(rental)
        }
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't update a rental with id={}. rental not found",
                id
            ),
        }),
    }
}

fn do_insert(rental: &BasketRental) {
    STORAGE.with(|service| service.borrow_mut().insert(rental.id, rental.clone()));
}

#[ic_cdk::update]
fn delete_basket_rental(id: u64) -> Result<BasketRental, Error> {
    match STORAGE.with(|service| service.borrow_mut().remove(&id)) {
        Some(rental) => Ok(rental),
        None => Err(Error::NotFound {
            msg: format!(
                "couldn't delete a rental with id={}. rental not found.",
                id
            ),
        }),
    }
}

#[ic_cdk::query]
fn show_list() -> Vec<BasketRental> {
    let mut result = Vec::new();

    for (_, rental) in STORAGE.with(|service| service.borrow().iter()) {
        match rental.tipe_lapangan {
            'A' => result.push(rental.clone()),
            'B' => result.push(rental.clone()),
            'C' => result.push(rental.clone()),
            _ => (),
        }
    }

    result
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
}

fn _get_basket_rental(id: &u64) -> Option<BasketRental> {
    STORAGE.with(|service| service.borrow().get(id))
}

ic_cdk::export_candid!();