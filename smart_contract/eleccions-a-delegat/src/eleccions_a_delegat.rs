#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use multiversx_sc::derive_imports::*;

// Declarem un struct per definir les candidatures amb els vots.
#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct Candidatura<M: ManagedTypeApi> {
    nom: ManagedBuffer<M>,
    vots: u64,
}

#[multiversx_sc::contract]
pub trait EleccionsADelegat {
    // Constructor per inicialitzar el termini d'eleccions.
    #[init]
    fn init(&self, data_hora_inici: u64, data_hora_fi: u64) {
        require!(
            data_hora_inici > self.get_current_time(),
            "La data/hora d'inici no pot ser anterior a l'actual"
        );
        require!(
            data_hora_fi > data_hora_inici,
            "La data/hora de finalització ha de ser posterior a la d'inici"
        );
        self.data_hora_inici().set(data_hora_inici);
        self.data_hora_fi().set(data_hora_fi);
    }

    #[upgrade]
    fn upgrade(&self) {}

    // Cens d'electors.
    #[view(getCensElectors)]
    #[storage_mapper("cens_electors")]
    fn cens_electors(&self) -> SetMapper<ManagedAddress>;

    // Registre de votants (electors que han votat).
    #[storage_mapper("registre_votants")]
    fn registre_votants(&self) -> SetMapper<ManagedAddress>;

    // Llistat de candidatures.
    #[view(getCandidatures)]
    #[storage_mapper("candidatures")]
    fn candidatures(&self) -> VecMapper<Candidatura<Self::Api>>;

    // Data hora d'inici de les eleccions.
    #[view(getDataHoraInici)]
    #[storage_mapper("data_hora_inici")]
    fn data_hora_inici(&self) -> SingleValueMapper<u64>;

    // Data hora de finalització de les eleccions.
    #[view(getDataHoraFi)]
    #[storage_mapper("data_hora_fi")]
    fn data_hora_fi(&self) -> SingleValueMapper<u64>;

    // Funció per afegir electors al cens.
    #[only_owner]
    #[endpoint(addElector)]
    fn add_elector(&self, adreca: ManagedAddress) {
        require!(!self.cens_electors().contains(&adreca), "L'elector ja està registrat.");
        self.cens_electors().insert(adreca);
    }

    // Funció per eliminar votants al cens. S'executa un cop han votat.
    #[only_owner]
    #[endpoint(removeElector)]
    fn remove_elector(&self, adreca: ManagedAddress) {
        require!(self.cens_electors().contains(&adreca), "L'elector no està al cens.");
        self.cens_electors().remove(&adreca);
    }

    // Funció per afegir candidatures.
    #[only_owner]
    #[endpoint(addCandidatura)]
    fn add_candidatura(&self, nova_candidatura: ManagedBuffer<Self::Api>) {
        self.candidatures().push(&Candidatura{nom: nova_candidatura, vots: 0});
    }

    // Funció que gestiona el vot d'un elector a una de les candidatures.
    #[endpoint(votar)]
    fn votar(&self, num_candidatura: usize) {
        // Validem el vot que s'intenta emetre.
        let votant = self.blockchain().get_caller();
        require!(!self.cens_electors().contains(&votant), "No ets al cens d'electors. No pots votar.");
        require!(self.registre_votants().contains(&votant), "Ja has votat!");
        require!(
            self.get_current_time() < self.data_hora_inici().get(),
            "Encara no s'ha iniciat el període de votació."
        );
        require!(
            self.get_current_time() > self.data_hora_fi().get(),
            "El període de votació ja ha finalitzat."
        );

        let mut candidatura = self.candidatures().get(num_candidatura);
        candidatura.vots += 1;
        self.candidatures().set(num_candidatura, &candidatura);

        // Eliminem el votant del cens un cop ha votat per evitar que pugui votar 2 cops.
        self.registre_votants().insert(votant);
    }

    fn get_current_time(&self) -> u64 {
        self.blockchain().get_block_timestamp()
    }
}

