#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_ok, assert_noop};
    use sp_core::H256;
    use frame_system::{EventRecord, Phase};
    use frame_support::traits::{OnFinalize, OnInitialize};
    use crate::pallet::{Event, Error};

    // Configurações de teste
    #[test]
    fn create_community_should_work() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let name = b"Kinera".to_vec();
            let description = b"Community for Kinera users".to_vec();
            let community_type = CommunityType::Public { monthly_fee: 10 };
            let icon = b"icon".to_vec();
            let social_user_name = b"kinera_user".to_vec();
            let proposal_reason = b"For all members".to_vec();

            assert_ok!(Communities::create_community(
                Origin::signed(creator),
                name.clone(),
                description.clone(),
                community_type.clone(),
                icon.clone(),
                social_user_name.clone(),
                proposal_reason.clone(),
            ));

            // Verificar o evento emitido
            System::assert_last_event(Event::CommunityCreated(1, creator).into());
        });
    }

    #[test]
    fn create_community_should_fail_if_name_too_long() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let name = vec![b'a'; 65]; // Excede o limite de 64
            let description = b"Community description".to_vec();
            let community_type = CommunityType::Private { monthly_fee: 0 };
            let icon = b"icon".to_vec();
            let social_user_name = b"user".to_vec();
            let proposal_reason = b"Good cause".to_vec();

            assert_noop!(
                Communities::create_community(
                    Origin::signed(creator),
                    name.clone(),
                    description.clone(),
                    community_type.clone(),
                    icon.clone(),
                    social_user_name.clone(),
                    proposal_reason.clone(),
                ),
                Error::<Test>::NameTooLong
            );
        });
    }

    #[test]
    fn submit_vote_should_work() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let voter = 2;
            let name = b"TestCommunity".to_vec();
            let description = b"Testing submit_vote".to_vec();
            let community_type = CommunityType::Public { monthly_fee: 0 };
            let icon = b"icon".to_vec();
            let social_user_name = b"voter".to_vec();
            let proposal_reason = b"Test voting".to_vec();

            // Criar comunidade
            assert_ok!(Communities::create_community(
                Origin::signed(creator),
                name.clone(),
                description.clone(),
                community_type.clone(),
                icon.clone(),
                social_user_name.clone(),
                proposal_reason.clone(),
            ));

            // Submeter voto
            assert_ok!(Communities::submit_vote(
                Origin::signed(voter),
                1, // community_id
                true,
                b"voter_name".to_vec(),
            ));

            // Verificar o evento emitido
            System::assert_last_event(Event::VoteSubmitted(1, true, voter).into());
        });
    }

    #[test]
    fn submit_vote_should_fail_if_already_voted() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let voter = 2;
            let name = b"Kinera".to_vec();
            let description = b"Community description".to_vec();
            let community_type = CommunityType::Public { monthly_fee: 0 };
            let icon = b"icon".to_vec();
            let social_user_name = b"user".to_vec();
            let proposal_reason = b"Good cause".to_vec();

            assert_ok!(Communities::create_community(
                Origin::signed(creator),
                name.clone(),
                description.clone(),
                community_type.clone(),
                icon.clone(),
                social_user_name.clone(),
                proposal_reason.clone(),
            ));

            assert_ok!(Communities::submit_vote(
                Origin::signed(voter),
                1, // community_id
                true,
                b"voter_name".to_vec(),
            ));

            assert_noop!(
                Communities::submit_vote(
                    Origin::signed(voter),
                    1,
                    false,
                    b"voter_name".to_vec(),
                ),
                Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn add_member_should_work() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let user = 2;
            let name = b"Kinera Community".to_vec();
            let description = b"A private community for testing".to_vec();
            let community_type = CommunityType::Public { monthly_fee: 0 };
            let icon = b"icon".to_vec();
            let social_user_name = b"user".to_vec();
            let proposal_reason = b"Member addition".to_vec();

            assert_ok!(Communities::create_community(
                Origin::signed(creator),
                name.clone(),
                description.clone(),
                community_type.clone(),
                icon.clone(),
                social_user_name.clone(),
                proposal_reason.clone(),
            ));

            assert_ok!(Communities::add_member(
                Origin::signed(user),
                1, // community_id
                b"user_social".to_vec(),
            ));

            // Verificar o evento emitido
            System::assert_last_event(Event::MemberAdded(user, 1).into());
        });
    }

    #[test]
    fn add_member_should_fail_if_already_member() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let user = 2;
            let name = b"Community".to_vec();
            let description = b"Private community".to_vec();
            let community_type = CommunityType::Private { monthly_fee: 0 };
            let icon = b"icon".to_vec();
            let social_user_name = b"user_social".to_vec();
            let proposal_reason = b"Test".to_vec();

            assert_ok!(Communities::create_community(
                Origin::signed(creator),
                name.clone(),
                description.clone(),
                community_type.clone(),
                icon.clone(),
                social_user_name.clone(),
                proposal_reason.clone(),
            ));

            // Primeiro adicionar
            assert_ok!(Communities::add_member(
                Origin::signed(user),
                1,
                b"user_social".to_vec(),
            ));

            // Tentar adicionar novamente
            assert_noop!(
                Communities::add_member(
                    Origin::signed(user),
                    1,
                    b"user_social".to_vec(),
                ),
                Error::<Test>::AlreadyMember
            );
        });
    }

    #[test]
    fn remove_member_should_work() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let user = 2;
            let name = b"CommunityTest".to_vec();
            let description = b"A testing community".to_vec();
            let community_type = CommunityType::Public { monthly_fee: 0 };
            let icon = b"icon".to_vec();
            let social_user_name = b"username".to_vec();
            let proposal_reason = b"Testing removal".to_vec();

            assert_ok!(Communities::create_community(
                Origin::signed(creator),
                name.clone(),
                description.clone(),
                community_type.clone(),
                icon.clone(),
                social_user_name.clone(),
                proposal_reason.clone(),
            ));

            // Adicionar membro para remover depois
            assert_ok!(Communities::add_member(
                Origin::signed(user),
                1, // community_id
                b"username_social".to_vec(),
            ));

            // Remover membro
            assert_ok!(Communities::remove_member(
                Origin::signed(creator),
                1, // community_id
                user,
                b"username_social".to_vec(),
            ));

            // Verificar o evento emitido
            System::assert_last_event(Event::MemberRemoved(user, 1).into());
        });
    }

    #[test]
    fn remove_member_should_fail_if_not_member() {
        new_test_ext().execute_with(|| {
            let creator = 1;
            let user = 2;
            let name = b"Kinera".to_vec();
            let description = b"Community description".to_vec();
            let community_type = CommunityType::Public { monthly_fee: 0 };
            let icon = b"icon".to_vec();
            let social_user_name = b"user_social".to_vec();
            let proposal_reason = b"Reason".to_vec();

            assert_ok!(Communities::create_community(
                Origin::signed(creator),
                name.clone(),
                description.clone(),
                community_type.clone(),
                icon.clone(),
                social_user_name.clone(),
                proposal_reason.clone(),
            ));

            // Tentativa de remover um usuário que não é membro
            assert_noop!(
                Communities::remove_member(
                    Origin::signed(creator),
                    1,
                    user,
                    b"user_social".to_vec(),
                ),
                Error::<Test>::NotMember
            );
        });
    }
}
