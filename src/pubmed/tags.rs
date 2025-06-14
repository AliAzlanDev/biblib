/// PubMed Format tags.
///
/// See https://pubmed.ncbi.nlm.nih.gov/help/#pubmed-format

#[allow(unused)]
#[allow(clippy::upper_case_acronyms)]
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum PubmedTag {
    /// AB - Abstract: English language abstract taken directly from the published article
    Abstract,
    /// AD - Affiliation: Author or corporate author addresses
    ///
    /// https://pubmed.ncbi.nlm.nih.gov/help/#ad
    Affiliation,
    /// AID - Article Identifier: Article ID values supplied by the publisher may include
    /// the pii (controlled publisher identifier), doi (digital object identifier), or
    /// book accession
    ArticleIdentifier,
    /// AU - Author: Authors
    ///
    /// https://pubmed.ncbi.nlm.nih.gov/help/#au
    Author,
    /// AUID - Author Identifier: Unique identifier associated with an author, corporate
    /// author, or investigator name
    ///
    /// https://pubmed.ncbi.nlm.nih.gov/help/#auid
    AuthorIdentifier,
    /// BTI - Book Title: Book Title
    BookTitle,
    /// \[...\] and so on...
    CopyrightInformation,
    CommentIn,
    CorporateAuthor,
    ConflictOfInterest,
    CommentOn,
    Chapter,
    CreateDate,
    CorrectedAndRepublishedFrom,
    CorrectedAndRepublishedIn,
    ContributionDate,
    CollectionTitle,
    CompletionDate,
    DatasetDescribedIn,
    DatasetUseReportedIn,
    DateOfElectronicPublication,
    PublicationDate,
    DateRevised,
    ExpressionOfConcernFor,
    ExpressionOfConcernIn,
    EntryDate,
    ErratumFor,
    ErratumIn,
    Editor,
    Edition,
    /// FAU - Full Author Name: Full author names
    FullAuthorName,
    /// FED - Full Editor Name: Full editor names
    FullEditorName,
    /// FIR - Full Investigator Name: Full investigator or collaborator names
    FullInvestigatorName,
    /// FPS - Full Personal Name as Subject: Full Personal Name of the subject of the article
    FullPersonalNameAsSubject,
    GeneralNote,
    GrantsAndFunding,
    GeneSymbol,
    Issue,
    Investigator,
    InvestigatorAffiliation,
    ISSN,
    ISBN,
    NLMUniqueID,
    FullJournalTitle,
    Language,
    LocationID,
    ModificationDate,
    MeSHTerms,
    MeSHDate,
    ManuscriptIdentifier,
    SubstanceName,
    OtherAbstract,
    OtherAbstractLanguage,
    OtherCopyrightInformation,
    OtherID,
    OriginalReportIn,
    OtherTerm,
    OtherTermOwner,
    Owner,
    Publisher,
    Pagination,
    PublicationHistoryStatusDate,
    PlaceOfPublication,
    PubmedCentralIdentifier,
    PMCRelease,
    PubMedUniqueIdentifier,
    PersonalNameAsSubject,
    PublicationStatus,
    PublicationType,
    NumberOfReferences,
    RetractionIn,
    ECorRNNumber,
    RetractionOf,
    RepublishedFrom,
    RepublishedIn,
    RetractedAndRepublishedIn,
    RetractedAndPublishedFrom,
    Subset,
    SpaceFlightMission,
    SecondarySourceID,
    Source,
    SummaryForPatientsIn,
    StatusTag,
    JournalTitleAbbreviation,
    Title,
    TransliteratedTitle,
    UpdateIn,
    UpdateOf,
    Volume,
    VolumeTitle,
}

impl PubmedTag {
    pub fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "AB" => Some(Self::Abstract),
            "AD" => Some(Self::Affiliation),
            "AID" => Some(Self::ArticleIdentifier),
            "AU" => Some(Self::Author),
            "AUID" => Some(Self::AuthorIdentifier),
            "BTI" => Some(Self::BookTitle),
            // TODO everything else...
            _ => None,
        }
    }
}
