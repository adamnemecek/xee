<!-- xsl:accept - OK to have an absent function that isn't called -->

<xsl:package
  version="3.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
  xmlns:xs="http://www.w3.org/2001/XMLSchema"
  xmlns:C="http://www.w3.org/xslt30/tests/accept"
  exclude-result-prefixes="xs C">
  
  <xsl:use-package
     name="http://www.w3.org/xslt30tests/accept-C"
     package-version="1.0.0">
    
    <xsl:override>
      <xsl:variable name="v1" as="xs:integer" select="22"/>
      
      <xsl:template name="t1" as="xs:integer">
        <xsl:param name="p1" as="xs:string"/>
        <xsl:sequence select="22"/>
      </xsl:template>
      
      <xsl:attribute-set name="a1">
        <xsl:attribute name="a" select="22"/>
      </xsl:attribute-set>    
      
    </xsl:override>
    
    <xsl:accept component="function" names="C:f1#1" visibility="hidden"/>
  </xsl:use-package>  
  
  
  <xsl:template name="main" visibility="public">
    <out xsl:use-attribute-sets="a1">
      <xsl:call-template name="t1">
        <xsl:with-param name="p1" select="string($v1)"/>
      </xsl:call-template>
    </out>
  </xsl:template>  
  

</xsl:package>   