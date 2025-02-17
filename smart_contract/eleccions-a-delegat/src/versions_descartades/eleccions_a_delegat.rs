#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use multiversx_sc::derive_imports::*;
//use multiversx_sc::imports::*;
//use multiversx_sc::codec;
//use multiversx_sc::proxy_imports::{TopDecode, TopEncode, type_abi};

// Declarem un struct per definir les candidatures amb els vots.
#[derive(TopEncode, TopDecode)]
#[type_abi]
pub struct Candidatura {
    nom: String,
    vots: u64,
}

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait EleccionsADelegat {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}

    // Implementació i gestió del cens de votants.
    #[storage_mapper("cens_votants")]
    fn cens_votants(&self) -> SetMapper<ManagedAddress>;

    #[only_owner]
    #[endpoint(afegirVotant)]
    fn afegir_votant(&self, adreca: ManagedAddress) {
        require!(!self.cens_votants().contains(&adreca), "El votant ja està registrat.");
        self.cens_votants().insert(adreca);
    }

    #[only_owner]
    #[endpoint(esborrarVotant)]
    fn esborrar_votant(&self, adreca: ManagedAddress) {
        require!(self.cens_votants().contains(&adreca), "El votant no està al cens.");
        self.cens_votants().remove(&adreca);
    }


    // Implementació i gestió del llistat de candidatures.
    #[endpoint(candidatures)]
    #[storage_mapper("candidatures")]
    fn candidatures(&self) -> VecMapper<Candidatura>;
    // Proposta de ChatGPT per eficiència i per possibilitar la modificació dels vots.
    // fn candidatures(&self) -> MapMapper<ManagedBuffer<Self::Api>, Candidatura<Self::Api>>;


    #[only_owner]
    #[endpoint(addCandidatura)]
    fn add_candidatura(&self, nova_candidatura: String) {
        self.candidatures().push(&Candidatura{nom: nova_candidatura, vots: 0});
    }

    #[endpoint(getCandidatures)]
    fn get_candidatures_public(&self) -> Vec<String> {
        let mut llista_noms: Vec<String> = Vec::new();
        for c in self.candidatures().iter() {
            llista_noms.push(c.nom);
        }

        llista_noms
    }

    #[endpoint(votar)]
    fn votar(&self, num_candidatura: usize) {
        let votant = self.blockchain().get_caller();
        require!(self.cens_votants().contains(&votant), "No tens permís per votar.");

        let mut candidatura = self.candidatures().get(num_candidatura);
        candidatura.vots += 1;
        self.candidatures().set(num_candidatura, &candidatura);

        // Eliminem el votant del cens un cop ha votat per evitar que pugui votar 2 cops.
        self.cens_votants().remove(&votant);
    }
}

