<?xml version="1.0" encoding="UTF-8"?>
<!--It is a static error if the
                        processor is not able to retrieve the resource identified by the URI
                           reference in the href attribute of
                              xsl:include or xsl:import
                        , or if the resource that is retrieved does not contain a
                        stylesheet module conforming to this specification.-->
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0">

<?error XTSE0165?>

  <xsl:import href="stdxmlfile.xml"/>


  <xsl:template name="main">
      <out>
         <xsl:apply-templates/>
      </out>
  </xsl:template>



</xsl:stylesheet>
