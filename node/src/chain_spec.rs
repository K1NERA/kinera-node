use kine_runtime::{
	AccountId, AuraConfig, BalancesConfig, GrandpaConfig, RuntimeGenesisConfig, Signature,
	SudoConfig, SystemConfig, WASM_BINARY,
	TagsModuleConfig, RankingListModuleConfig,
	CategoryStringLimit, TagStringLimit, MaxTags, 
	RankingStringLimit,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	BoundedVec,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> RuntimeGenesisConfig {
	
	//TODO optimize
	//TODO festivals and movies use copies of the ranking list's tags/categories
	
	let rl_all_time_best: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"All Times, Best Films".as_bytes().to_vec().try_into().unwrap(),
		"The best films on the platform.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);
	
	let rl_best_south_america: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best South America Cinema".as_bytes().to_vec().try_into().unwrap(),
		"The best films from South America.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);
	
	let rl_best_asian_fiction: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Asiatic Fiction".as_bytes().to_vec().try_into().unwrap(),
		"The best Asian Fiction movies.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);
	
	let rl_best_crypto: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Crypto Youtuber/TikTok Channels, Online".as_bytes().to_vec().try_into().unwrap(),
		"The best content regarding Web3 and Crypto.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);
	
	let rl_documentaries_and_fiction: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Documentaries and Fiction, Online".as_bytes().to_vec().try_into().unwrap(),
		"The best films on the platform.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);
	
	// initialize the vec with the pre-built tuples
	let initial_ranking_lists = vec![
		rl_all_time_best, 
		rl_best_south_america,
		rl_best_asian_fiction,
		rl_best_crypto,
		rl_documentaries_and_fiction,
	];
	


// Tags Ranking Lists

		let rl_cinema_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Fiction".as_bytes().to_vec().try_into().unwrap(),
				"Science Fiction".as_bytes().to_vec().try_into().unwrap(),
				"Drama".as_bytes().to_vec().try_into().unwrap(),
				"Documentary".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let rl_stars_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Influencers".as_bytes().to_vec().try_into().unwrap(),
				"Educator".as_bytes().to_vec().try_into().unwrap(),
				"Pivots".as_bytes().to_vec().try_into().unwrap(),
				"Web3".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let rl_just_fun_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Art".as_bytes().to_vec().try_into().unwrap(),
				"Dance".as_bytes().to_vec().try_into().unwrap(),
				"Disasters".as_bytes().to_vec().try_into().unwrap(),
				"Kids".as_bytes().to_vec().try_into().unwrap(),
				"Sports".as_bytes().to_vec().try_into().unwrap(),
				"Products".as_bytes().to_vec().try_into().unwrap(),
				"Professionals".as_bytes().to_vec().try_into().unwrap(),
				"News".as_bytes().to_vec().try_into().unwrap(),
				"Services".as_bytes().to_vec().try_into().unwrap(),
				"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();
			
		let rl_we_festivals_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Information".as_bytes().to_vec().try_into().unwrap(),
				"Offbeat Films/Marginal".as_bytes().to_vec().try_into().unwrap(),
				"Experimental Cinema".as_bytes().to_vec().try_into().unwrap(),
				"Video Art".as_bytes().to_vec().try_into().unwrap(),
				"Video Clips".as_bytes().to_vec().try_into().unwrap(),
				"We Festivals Others".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();
			
		let rl_artificial_intelligence_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"AI Drama".as_bytes().to_vec().try_into().unwrap(),
				"AI Documentary".as_bytes().to_vec().try_into().unwrap(),
				"AI Fiction".as_bytes().to_vec().try_into().unwrap(),
				"AI Fake".as_bytes().to_vec().try_into().unwrap(),
				"AI Science Fiction".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let rl_gaming_streamers_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Action/Adventure".as_bytes().to_vec().try_into().unwrap(),
				"Role-Playing".as_bytes().to_vec().try_into().unwrap(),
				"Strategy".as_bytes().to_vec().try_into().unwrap(),
				"Sports".as_bytes().to_vec().try_into().unwrap(),
				"Puzzle".as_bytes().to_vec().try_into().unwrap(),
				"Simulation".as_bytes().to_vec().try_into().unwrap(),
				"Racing".as_bytes().to_vec().try_into().unwrap(),
				"Fighting".as_bytes().to_vec().try_into().unwrap(),
				"Platformer".as_bytes().to_vec().try_into().unwrap(),
				"MMO (Massive Multiplayer Online)".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();



	// Tags Moderation

		let mod_violence_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_discrimination_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_lack_of_consent_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_impersonation_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_terrorism_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_copyright_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_misinformation_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_pornography_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_extreme_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_naming_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

		let mod_categorization_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Movie".as_bytes().to_vec().try_into().unwrap(),
				"Festival".as_bytes().to_vec().try_into().unwrap(),
				"RankingList".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();


	// setup the final map with all categories and tags by type
	let initial_categories_and_tags: Vec <(
		(BoundedVec<u8, CategoryStringLimit>, BoundedVec<u8, CategoryStringLimit>),
		BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
	)> = vec![
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Stars".as_bytes().to_vec().try_into().unwrap()), rl_stars_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Artifitial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags.clone()),
		(("Movie".as_bytes().to_vec().try_into().unwrap(), "Gaming/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags.clone()),

		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Stars".as_bytes().to_vec().try_into().unwrap()), rl_stars_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Artifitial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags.clone()),
		(("Festival".as_bytes().to_vec().try_into().unwrap(), "Gaming/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags.clone()),

		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Stars".as_bytes().to_vec().try_into().unwrap()), rl_stars_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Artifitial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Gaming/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags),

		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Violence".as_bytes().to_vec().try_into().unwrap()), mod_violence_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Discrimination".as_bytes().to_vec().try_into().unwrap()), mod_discrimination_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "LackOfConsent".as_bytes().to_vec().try_into().unwrap()), mod_lack_of_consent_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Impersonation".as_bytes().to_vec().try_into().unwrap()), mod_impersonation_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Terrorism".as_bytes().to_vec().try_into().unwrap()), mod_terrorism_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Copyright".as_bytes().to_vec().try_into().unwrap()), mod_copyright_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Misinformation".as_bytes().to_vec().try_into().unwrap()), mod_misinformation_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Pornography".as_bytes().to_vec().try_into().unwrap()), mod_pornography_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Extreme".as_bytes().to_vec().try_into().unwrap()), mod_extreme_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Naming".as_bytes().to_vec().try_into().unwrap()), mod_naming_tags),
		(("Moderation".as_bytes().to_vec().try_into().unwrap(), "Categorization".as_bytes().to_vec().try_into().unwrap()), mod_categorization_tags),
	];
	
	RuntimeGenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		aura: AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		},
		grandpa: GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
			..Default::default()
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),

		tags_module: TagsModuleConfig {
			category_to_tag_map: initial_categories_and_tags.iter().cloned().map(|x| x).collect(),
		},

		ranking_list_module: RankingListModuleConfig {
			default_ranking_lists: initial_ranking_lists.iter().cloned().map(|x| x).collect(),
		},
	}
}
