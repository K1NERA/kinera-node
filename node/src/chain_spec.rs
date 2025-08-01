use kine_runtime::{
	AccountId, AuraConfig, BalancesConfig, GrandpaConfig, RuntimeGenesisConfig, Signature,
	SudoConfig, SystemConfig, WASM_BINARY,
	TagsModuleConfig, RankingListModuleConfig,
	CategoryStringLimit, TagStringLimit, MaxTags, 
	RankingStringLimit,CommunitiesModuleConfig
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	BoundedVec,
};
use sp_core::crypto::Ss58Codec;
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
		"Kinera",
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
		"All Time, Best Films".as_bytes().to_vec().try_into().unwrap(),
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
		"Best Asian Fiction".as_bytes().to_vec().try_into().unwrap(),
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
		"Best Crypto Advocates and Educators".as_bytes().to_vec().try_into().unwrap(),
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
		"Best Documentaries Online".as_bytes().to_vec().try_into().unwrap(),
		"The best films on the platform.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);


  let crypto_advocates: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Crypto Advocates and Educators".as_bytes().to_vec().try_into().unwrap(),
		"Content creators who inspire users while explaining the principals of the technology.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let crypto_advocates: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Crypto Podcasts".as_bytes().to_vec().try_into().unwrap(),
		"Generalist crypto podcasts featuring the best scientists, thinkers, projects and ideas.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

 

  let venture_capital: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Venture Capital and Institutional Content Creators".as_bytes().to_vec().try_into().unwrap(),
		"institutional projects ventures capi:  Some of the best discussions are organised by investment companies, foundations or companies connected to projects. Here we list and rank the institutional content.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let blockchain_artificial: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"The best content About Web3 and Crypto".as_bytes().to_vec().try_into().unwrap(),
		"The intersection between Decentralised Artificial Intelligence and Blockchain is in its infancy.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let crypto_economic_analysts: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Top Traders and Analysts".as_bytes().to_vec().try_into().unwrap(),
		"The place for market analysts, covering market analysis and market trends.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

 

  let institutional_content: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Institutional Content".as_bytes().to_vec().try_into().unwrap(),
		"Some of the best discussions are organised by investment companies, foundations or companies connected to projects. Here we list and rank the institutional content. " 
    .as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let degen_tubers: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Top Degen Tubers".as_bytes().to_vec().try_into().unwrap(),
		"Tubers, analysts and traders who frequently engage in highly speculative and risky activities. Only out of the box ideas, please." 
    .as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let bet_ai_author: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best AI Authors".as_bytes().to_vec().try_into().unwrap(),
		"Let's start ranking creators, directors, auteurs and AI film experimentalists on YouTube." 
    .as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let social_media: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Social Media Content, Advertising and Marketing".as_bytes().to_vec().try_into().unwrap(),
		"AI-generated videos for self- or institutional promotion, serious or humorous." 
    .as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let deep_fake: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Deepfakes".as_bytes().to_vec().try_into().unwrap(),
		"Films that use AI and machine learning technologies to create highly realistic and convincing videos that make it appear as though people are saying or doing things they did not actually say or do." 
    .as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let experimental_and_art_films: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Experimental Experimental and Art Films".as_bytes().to_vec().try_into().unwrap(),
		"Videos where creators move beyond dream and reality." 
    .as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let podcasts_all_tools: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"AI Tools for Audio and Video Production".as_bytes().to_vec().try_into().unwrap(),
		"The top videos showcasing AI production tools for content creation." 
    .as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let best_stand_up_comedian_channels: (
    BoundedVec<u8, RankingStringLimit>,
    BoundedVec<u8, RankingStringLimit>,
    u64,
    BoundedVec<(
        BoundedVec<u8, CategoryStringLimit>,
        BoundedVec<u8, TagStringLimit>,
    ), MaxTags>,
  ) = (
    "Best Stand-Up Comedian Videos".as_bytes().to_vec().try_into().unwrap(),
    "The Best Stand-Up Comedians and Videos."
    .as_bytes().to_vec().try_into().unwrap(),
    4800u64.into(),
    TryInto::try_into(vec![(
        "Just FUN".as_bytes().to_vec().try_into().unwrap(),
        "Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
    )]).unwrap(),
  );

  let comedy_best_single_videos_online: (
    BoundedVec<u8, RankingStringLimit>,
    BoundedVec<u8, RankingStringLimit>,
    u64,
    BoundedVec<(
        BoundedVec<u8, CategoryStringLimit>,
        BoundedVec<u8, TagStringLimit>,
    ), MaxTags>,
  ) = (
    "Comedy Best Single Videos Online".as_bytes().to_vec().try_into().unwrap(),
    "Comedy sketches, spoofs and hidden camera shorts."
    .as_bytes().to_vec().try_into().unwrap(),
    4800u64.into(),
    TryInto::try_into(vec![(
        "Just FUN".as_bytes().to_vec().try_into().unwrap(),
        "Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
    )]).unwrap(),
  );


  let best_video_memes: (
    BoundedVec<u8, RankingStringLimit>,
    BoundedVec<u8, RankingStringLimit>,
    u64,
    BoundedVec<(
        BoundedVec<u8, CategoryStringLimit>,
        BoundedVec<u8, TagStringLimit>,
    ), MaxTags>,
  ) = (
    "Best Video Memes".as_bytes().to_vec().try_into().unwrap(),
    "All video memes in one place."
    .as_bytes().to_vec().try_into().unwrap(),
    4800u64.into(),
    TryInto::try_into(vec![(
        "Just FUN".as_bytes().to_vec().try_into().unwrap(),
        "Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
    )]).unwrap(),
  );


  let funny_with_animals: (
    BoundedVec<u8, RankingStringLimit>,
    BoundedVec<u8, RankingStringLimit>,
    u64,
    BoundedVec<(
        BoundedVec<u8, CategoryStringLimit>,
        BoundedVec<u8, TagStringLimit>,
    ), MaxTags>,
  ) = (
    "Best Videos on Politics and Politicians".as_bytes().to_vec().try_into().unwrap(),
    "Let's check out the top performers in politics and politicians!"
    .as_bytes().to_vec().try_into().unwrap(),
    4800u64.into(),
    TryInto::try_into(vec![(
        "Just FUN".as_bytes().to_vec().try_into().unwrap(),
        "Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
    )]).unwrap(),
  );


  let all_time_best_films: (
    BoundedVec<u8, RankingStringLimit>,
    BoundedVec<u8, RankingStringLimit>,
    u64,
    BoundedVec<(
        BoundedVec<u8, CategoryStringLimit>,
        BoundedVec<u8, TagStringLimit>,
    ), MaxTags>,
  ) = (
      "All Time Best Films".as_bytes().to_vec().try_into().unwrap(),
      "Historical landmarks and life-changing fiction and documentaries."
      .as_bytes().to_vec().try_into().unwrap(),
      4800u64.into(),
      TryInto::try_into(vec![(
          "Cinema".as_bytes().to_vec().try_into().unwrap(),
          "All Genres".as_bytes().to_vec().try_into().unwrap(),
      )]).unwrap(),
  );


  let all_time_best_asiatic_fiction: (
    BoundedVec<u8, RankingStringLimit>,
    BoundedVec<u8, RankingStringLimit>,
    u64,
    BoundedVec<(
        BoundedVec<u8, CategoryStringLimit>,
        BoundedVec<u8, TagStringLimit>,
    ), MaxTags>,
  ) = (
      "All Time Best Asiatic Fiction".as_bytes().to_vec().try_into().unwrap(),
      "Cinema from India, China, Pakistan, Thailand, Taiwan, South Korea, Japan, Iran, etc."
      .as_bytes().to_vec().try_into().unwrap(),
      4800u64.into(),
      TryInto::try_into(vec![(
          "Cinema".as_bytes().to_vec().try_into().unwrap(),
          "Asiatic Fiction".as_bytes().to_vec().try_into().unwrap(),
      )]).unwrap(),
  );

  let the_best_from_south_america: (
    BoundedVec<u8, RankingStringLimit>,
    BoundedVec<u8, RankingStringLimit>,
    u64,
    BoundedVec<(
        BoundedVec<u8, CategoryStringLimit>,
        BoundedVec<u8, TagStringLimit>,
    ), MaxTags>,
  ) = (
      "The Best From South America".as_bytes().to_vec().try_into().unwrap(),
      "Documentary and fiction."
      .as_bytes().to_vec().try_into().unwrap(),
      4800u64.into(),
      TryInto::try_into(vec![(
          "Cinema".as_bytes().to_vec().try_into().unwrap(),
          "South American".as_bytes().to_vec().try_into().unwrap(),
      )]).unwrap(),
  );

  let life_changing_documentary_films_online: (
    BoundedVec<u8, RankingStringLimit>,
    BoundedVec<u8, RankingStringLimit>,
    u64,
    BoundedVec<(
        BoundedVec<u8, CategoryStringLimit>,
        BoundedVec<u8, TagStringLimit>,
    ), MaxTags>,
  ) = (
      "Life-changing Documentary Films Online".as_bytes().to_vec().try_into().unwrap(),
      "Full-length documentaries online."
      .as_bytes().to_vec().try_into().unwrap(),
      4800u64.into(),
      TryInto::try_into(vec![(
          "Cinema".as_bytes().to_vec().try_into().unwrap(),
          "Documentaries".as_bytes().to_vec().try_into().unwrap(),
      )]).unwrap(),
  );

  let best_shorts_all_genre_world_wide: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Shorts, All Genre, World Wide".as_bytes().to_vec().try_into().unwrap(),
		"Small films are beautiful.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);
	
  let best_comic_kids: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Comic Kids and Animal Videos".as_bytes().to_vec().try_into().unwrap(),
		"Best of Funny Kids Videos from around the world.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  	
  let best_comedy_channels: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Comedy Channels".as_bytes().to_vec().try_into().unwrap(),
		"Best Comedy Channels on YouTube.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);

  let best_podcasts_about_ai: (
		BoundedVec<u8, RankingStringLimit>,
		BoundedVec<u8, RankingStringLimit>,
		u64,
		BoundedVec<(
			BoundedVec<u8, CategoryStringLimit>,
			BoundedVec<u8, TagStringLimit>,
		), MaxTags>,
	) = (
		"Best Podcasts About AI".as_bytes().to_vec().try_into().unwrap(),
		"Discussions about the latest developments and the future of AI.".as_bytes().to_vec().try_into().unwrap(),
		4800u64.into(),
		TryInto::try_into(vec![(
			"Just FUN".as_bytes().to_vec().try_into().unwrap(),
			"Just FUN Others".as_bytes().to_vec().try_into().unwrap(),
		)]).unwrap(),
	);


  
	

	// initialize the vec with the pre-built tuples
	let initial_ranking_lists = vec![
		rl_all_time_best, 
    best_podcasts_about_ai,
    best_comedy_channels,
		rl_best_south_america,
		rl_best_asian_fiction,
		rl_best_crypto,
		rl_documentaries_and_fiction,
    crypto_advocates,
    venture_capital,
    blockchain_artificial,
    crypto_economic_analysts,
    institutional_content,
    degen_tubers,
    best_stand_up_comedian_channels,
    comedy_best_single_videos_online,
    best_video_memes,
    funny_with_animals,
    all_time_best_films,
    all_time_best_asiatic_fiction,
    the_best_from_south_america,
    life_changing_documentary_films_online,
    best_shorts_all_genre_world_wide,
    best_comic_kids,
    bet_ai_author,
    social_media,
    deep_fake,
    experimental_and_art_films,
    podcasts_all_tools
	];


  let initial_community: (
    BoundedVec<u8, CategoryStringLimit>, // Nome da comunidade
    BoundedVec<u8, TagStringLimit>,      // Descrição da comunidade
    AccountId                            // Dono da comunidade
) = (
    "Beta testers".as_bytes().to_vec().try_into().unwrap(), // Nome
    "Community focused on creators who participated in the Kinera beta phase".as_bytes().to_vec().try_into().unwrap(), // Descrição
    AccountId::from_ss58check("5EZ2LivaBtyaa39CVtZTtUbVPYb4buiAMaYCecMpN1rKuKs9").unwrap() // Dono
);
let initial_communities = vec![
    initial_community // Pode adicionar mais comunidades aqui se necessário no futuro
];




// Tags Ranking Lists

		let rl_cinema_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Drama".as_bytes().to_vec().try_into().unwrap(),
				"Documentary".as_bytes().to_vec().try_into().unwrap(),
				"Fiction".as_bytes().to_vec().try_into().unwrap(),
        "Horror".as_bytes().to_vec().try_into().unwrap(),
        "Influencers".as_bytes().to_vec().try_into().unwrap(),
        "Romantic".as_bytes().to_vec().try_into().unwrap(),
        "Science Fiction".as_bytes().to_vec().try_into().unwrap(),
        "Thriller".as_bytes().to_vec().try_into().unwrap(),
        "Other/All".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

    let crypto_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
    = vec![
      "AI".as_bytes().to_vec().try_into().unwrap(),
      "Degen".as_bytes().to_vec().try_into().unwrap(),
      "Economy".as_bytes().to_vec().try_into().unwrap(),
      "Influencers".as_bytes().to_vec().try_into().unwrap(),
      "Interview".as_bytes().to_vec().try_into().unwrap(),
      "Other".as_bytes().to_vec().try_into().unwrap(),
      "Traders".as_bytes().to_vec().try_into().unwrap(),
     
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
        "Experimental Cinema".as_bytes().to_vec().try_into().unwrap(),
				"Information".as_bytes().to_vec().try_into().unwrap(),
				"Offbeat/Marginal Cinema".as_bytes().to_vec().try_into().unwrap(),
        "Influencers".as_bytes().to_vec().try_into().unwrap(),
				"Video Art".as_bytes().to_vec().try_into().unwrap(),
				"Video Clip".as_bytes().to_vec().try_into().unwrap(),
				"Others".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();
			
		let media_makers_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
			= vec![
				"Activists".as_bytes().to_vec().try_into().unwrap(),
				"Artists and Designers".as_bytes().to_vec().try_into().unwrap(),
				"Educators".as_bytes().to_vec().try_into().unwrap(),
				"Entertainers".as_bytes().to_vec().try_into().unwrap(),
				"Food and Cooking".as_bytes().to_vec().try_into().unwrap(),
				"Lifestyle Influencers".as_bytes().to_vec().try_into().unwrap(),
				"Tech and Gadget".as_bytes().to_vec().try_into().unwrap(),
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
        "Fighting".as_bytes().to_vec().try_into().unwrap(),
        "Influencers".as_bytes().to_vec().try_into().unwrap(),
        "MMO (Massive Multiplayer Online)".as_bytes().to_vec().try_into().unwrap(),
        "Platformer".as_bytes().to_vec().try_into().unwrap(),
				"Role-Playing".as_bytes().to_vec().try_into().unwrap(),
        "Simulation".as_bytes().to_vec().try_into().unwrap(),
        "Sports".as_bytes().to_vec().try_into().unwrap(),
		].try_into().unwrap();

    let rl_strategy_tags : BoundedVec<BoundedVec<u8, TagStringLimit>, MaxTags>
    = vec![
      "Disasters".as_bytes().to_vec().try_into().unwrap(),
      "Fiction".as_bytes().to_vec().try_into().unwrap(),
      "Influencers".as_bytes().to_vec().try_into().unwrap(),
      "Kids and Animals".as_bytes().to_vec().try_into().unwrap(),
      "Memes".as_bytes().to_vec().try_into().unwrap(),
      "Politics".as_bytes().to_vec().try_into().unwrap(),
      "Stand up".as_bytes().to_vec().try_into().unwrap(),
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
    (("Movie".as_bytes().to_vec().try_into().unwrap(), "Artificial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags.clone()),
	(("Movie".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags.clone()),
	//	(("Movie".as_bytes().to_vec().try_into().unwrap(), "Stars".as_bytes().to_vec().try_into().unwrap()), rl_stars_tags.clone()),
    (("Movie".as_bytes().to_vec().try_into().unwrap(), "Crypto".as_bytes().to_vec().try_into().unwrap()), crypto_tags.clone()),
    (("Movie".as_bytes().to_vec().try_into().unwrap(), "Gamers/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags.clone()),
    (("Movie".as_bytes().to_vec().try_into().unwrap(), "Just Fun".as_bytes().to_vec().try_into().unwrap()), rl_strategy_tags.clone()),
    (("Movie".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags.clone()),

		//(("Movie".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags.clone()),
		

	(("Festival".as_bytes().to_vec().try_into().unwrap(), "Media Makers".as_bytes().to_vec().try_into().unwrap()), media_makers_tags.clone()),
    (("Festival".as_bytes().to_vec().try_into().unwrap(), "Artificial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags.clone()),
	(("Festival".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags.clone()),
    (("Festival".as_bytes().to_vec().try_into().unwrap(), "Crypto".as_bytes().to_vec().try_into().unwrap()), crypto_tags.clone()),
    (("Festival".as_bytes().to_vec().try_into().unwrap(), "Gamers/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags.clone()),
    (("Festival".as_bytes().to_vec().try_into().unwrap(), "Just Fun".as_bytes().to_vec().try_into().unwrap()), rl_strategy_tags.clone()),
    (("Festival".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags.clone()),

		//(("Festival".as_bytes().to_vec().try_into().unwrap(), "Stars".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags.clone()),
		//(("Festival".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags.clone()),

		

    (("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Artificial Intelligence".as_bytes().to_vec().try_into().unwrap()), rl_artificial_intelligence_tags),
		(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Cinema".as_bytes().to_vec().try_into().unwrap()), rl_cinema_tags),
    (("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Crypto".as_bytes().to_vec().try_into().unwrap()), crypto_tags.clone()),
    (("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Gamers/Streamers".as_bytes().to_vec().try_into().unwrap()), rl_gaming_streamers_tags),
    (("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Just Fun".as_bytes().to_vec().try_into().unwrap()), rl_strategy_tags.clone()),
    (("Ranking List".as_bytes().to_vec().try_into().unwrap(), "We Festivals".as_bytes().to_vec().try_into().unwrap()), rl_we_festivals_tags),

	//	(("Ranking List".as_bytes().to_vec().try_into().unwrap(), "Just FUN".as_bytes().to_vec().try_into().unwrap()), rl_just_fun_tags),
		
		
		

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

    communities_module: CommunitiesModuleConfig {
      initial_community_name: b"Beta Testers".to_vec(),
      initial_community_description: b"Community focused on creators who participated in Kinera's beta phase.".to_vec(),
	  initial_short_description: b"Community focused on creators who participated in Kinera's beta phase.".to_vec(),
      initial_reason: b" <h1>Beta Testers Community</h1> - Focused on Creators from Kinera's Beta Phase The Beta Testers community is dedicated to creators who participated in Kinera's beta phase. The goal is to foster collaboration, allowing pioneers to shape and improve the platform. Members can share experiences, provide feedback, and contribute to future features. This exclusive community ensures a space for meaningful discussions and continued commitment to Kineras development.
      ".to_vec(),
      initial_social_owner: b"Kinera Beta Testers".to_vec(),
	  initial_category: b"Kinera Beta Testers".to_vec(),
      initial_community_icon: b"https://i.ibb.co/3NDg07r/DALL-E-2024-09-09-11-37-44-A-logo-for-a-community-called-Beta-Testers-featuring-a-stylized-symbol-in.webp".to_vec(),
      initial_community_owner: Some(AccountId::from_ss58check("5H6RMcrTVhCAwu3nb82ckyHGWL1Ab1aZRiZcJYutAkif88xE").unwrap()),
    },

		ranking_list_module: RankingListModuleConfig {
			default_ranking_lists: initial_ranking_lists.iter().cloned().map(|x| x).collect(),
		},
	}
}
